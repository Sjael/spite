use std::{time::{Instant, Duration}, collections::HashMap};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{ui::ui_bundles::{RootUI, RespawnText, Killfeed, kill_notification}, stats::{Attribute, Health, Gold, Experience}, player::{IncomingDamageLog, Player, SpawnEvent}, GameState};



pub struct GameManagerPlugin;
impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameModeDetails::default());
        app.insert_resource(Player::new(1507));
        app.register_type::<Bounty>();
        app.add_state::<CharacterState>();
        app.add_event::<DeathEvent>();

        app.add_systems((
            check_deaths,
            increment_bounty,
            handle_respawning,
            tick_respawn_ui,
        ).in_set(OnUpdate(GameState::InGame)));
    }
}

fn swap_cameras(
    spectating: Res<Spectating>,
    mut spawn_events: EventReader<SpawnEvent>,
){
    if spectating.is_changed(){

    }
}


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CharacterState{
    Alive,
    #[default]
    Dead,
}

#[derive(Component)]
pub struct SpectatorCam;

#[derive(Resource)]
pub struct Spectating(pub Entity);

#[derive(Default)]
pub enum GameMode{
    #[default]
    Arena,
    Tutorial,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Team(pub u32);

#[derive(Resource)]
pub struct GameModeDetails{
    pub mode: GameMode,
    pub start_timer: i32,
    pub respawns: HashMap<Player, Timer>,
}

impl Default for GameModeDetails{
    fn default() -> Self {
        Self { 
            // Pre-game timer
            start_timer: -65,
            respawns: HashMap::new(),
            mode: GameMode::default(),
        }
    }
}

#[derive(Component)]
pub struct Fountain;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Bounty{
    pub xp: f32,
    pub gold: f32,
}

impl Default for Bounty{
    fn default() -> Self {
        Self{
            xp: 200.0,
            gold: 250.0,
        }
    }
}

fn handle_respawning(
    time: Res<Time>,
    mut gamemodedetails: ResMut<GameModeDetails>,
    mut spawn_events: EventWriter<SpawnEvent>,
){
    //let theredeemed = commands.spawn(()).id();
    // need to combine the loading and respawn system to go from an event
    // need character enum probably
    
    gamemodedetails.respawns.retain(|redeemed, timer| {
        timer.tick(time.delta());
        if timer.finished(){
            spawn_events.send(SpawnEvent{
                player: redeemed.clone(), // change to character type (minion, player, jungle camp)
                transform: Transform {
                    translation: Vec3::new(2.0, 0.5, 8.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..default()
                },
            });
        }
        !timer.finished()
    });
}

fn tick_respawn_ui(
    mut death_timer: Query<(&mut Visibility, &mut Text), With<RespawnText>>,
    gamemodedetails: ResMut<GameModeDetails>,
    local_player: Res<Player>,
){
    let Ok((mut vis, mut respawn_text)) = death_timer.get_single_mut() else { return };
    // match to player id later
    if let Some(timer) = gamemodedetails.respawns.get(&*local_player) {
        *vis = Visibility::Visible;
        let new_text = (timer.duration().as_secs() as f32 - timer.elapsed_secs()).floor() as u64;
        respawn_text.sections[1].value = new_text.to_string();
    } else{
        *vis = Visibility::Hidden;
    }
}

pub struct DeathEvent{
    pub player: Player,
}

fn check_deaths(
    mut commands: Commands,
    mut the_damned: Query<(Entity, &Attribute<Health>, Option<&mut Bounty>, &IncomingDamageLog, &Player)>, // Changed<IncomingDamageLog>
    mut the_victors: Query<(Entity, &mut Attribute<Gold>, &mut Attribute<Experience>)>,// make optional so creep can kill
    mut gamemodedetails: ResMut<GameModeDetails>,
    ui: Query<Entity, With<RootUI>>,
    mut character_state_next: ResMut<NextState<CharacterState>>,
    mut death_events: EventWriter<DeathEvent>,
){
    const TIME_FOR_KILL_CREDIT: u64 = 30;
    for (guy, hp, bounty, damagelog, player) in the_damned.iter_mut(){
        if *hp.amount() <= 0.0 {
            character_state_next.set(CharacterState::Dead);
            commands.entity(guy).despawn_recursive();
            if let Ok(ui) = ui.get_single(){
                commands.entity(ui).despawn_recursive(); // simply spectate something else in new ui system
            }
            gamemodedetails.respawns.insert(
                player.clone(), // send ACTUAL character info
                Timer::new(Duration::from_secs(8), TimerMode::Once), // figure respawn time later
            );
            death_events.send(DeathEvent{
                player: player.clone(),
            });

            if let Some(mut bounty) = bounty{
                for instance in damagelog.map.iter(){
                    if Instant::now().duration_since(instance.when) > Duration::from_secs(TIME_FOR_KILL_CREDIT) {
                        if let Ok((_awardee, mut gold, mut xp)) = the_victors.get_mut(instance.attacker){
                            *gold += bounty.gold;
                            *xp += bounty.xp;
                        }
                    }
                }
                *bounty = Bounty::default();
            }
        }
    }
}

// Make only increment when damageable? Aegis before guarenteed death is meta now LMAO
fn increment_bounty(
    mut the_notorious: Query<&mut Bounty>,
    time: Res<Time>,
){
    for mut wanted in the_notorious.iter_mut(){
        wanted.gold += 2.0 * time.delta_seconds();
        wanted.xp += 4.0 * time.delta_seconds();
    }
}


// Collision Grouping Flags
bitflags::bitflags! {
    pub struct Groups: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const ABILITY = 1 << 2;
        const GROUND = 1 << 3;
        const FLUFF = 1 << 4;

        const PLAYER_FILTER = Groups::PLAYER.bits() | Groups::TERRAIN.bits()| Groups::GROUND.bits();
        const TERRAIN_FILTER = Groups::PLAYER.bits() | Groups::TERRAIN.bits();
        // Make this interact with Terrain and other Abilities when we want cool interactions or no pen walls
        const ABILITY_FILTER = Groups::PLAYER.bits();
    }
}

pub const PLAYER_GROUPING: CollisionGroups = CollisionGroups::new(
        Group::from_bits_truncate(Groups::PLAYER.bits()), 
        Group::from_bits_truncate(Groups::PLAYER_FILTER.bits())
    );
pub const TERRAIN_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::TERRAIN.bits()), 
    Group::from_bits_truncate(Groups::TERRAIN_FILTER.bits())
);
pub const ABILITY_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::ABILITY.bits()), 
    Group::from_bits_truncate(Groups::ABILITY_FILTER.bits())
);