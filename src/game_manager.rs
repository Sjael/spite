use std::{time::{Instant, Duration}, collections::HashMap};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    ui::ui_bundles::{PlayerUI, RespawnText, RespawnHolder,}, 
    stats::{Attribute, Health, Gold, Experience}, 
    player::{IncomingDamageLog, Player, SpawnEvent, cast_ability, Reticle}, GameState, 
    ability::{Ability, ability_bundles::*, TargetsInArea, TargetsToEffect, EffectApplyType, OnEnterEffect, Ticks, Tags, TagInfo, homing::Homing, TargetsHit}, 
};



pub struct GameManagerPlugin;
impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameModeDetails::default());
        app.insert_resource(Player::new(1507));
        app.register_type::<Bounty>();
        app.add_state::<CharacterState>();
        
        app.add_event::<DeathEvent>();
        app.add_event::<CastEvent>();

        app.add_systems((
            check_deaths,
            increment_bounty,
            handle_respawning,
            show_respawn_ui,
            tick_respawn_ui,
            place_ability.after(cast_ability),
        ).in_set(OnUpdate(GameState::InGame)));
    }
}



#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CharacterState{
    Alive,
    #[default]
    Dead,
}


#[derive(Default)]
pub enum GameMode{
    #[default]
    Arena,
    Tutorial,
}


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


// TODO implement player id for multiplayer, and its child reticle 
fn place_ability(
    mut commands: Commands,
    mut cast_events: EventReader<CastEvent>,
    caster: Query<&GlobalTransform>,
    reticle: Query<&GlobalTransform, With<Reticle>>,
    player: Query<Entity, With<Player>>,
){
    let Ok(reticle_transform) = reticle.get_single() else {return};
    let Ok(player_e) = player.get_single() else {return};
    for event in cast_events.iter() {
        let Ok(caster_transform) = caster.get(event.caster) else {return};

        // Get ability-specific components
        let spawned = match event.ability{
            Ability::Frostbolt => {
                let blueprint = FrostboltInfo::default();
                blueprint.fire(&mut commands, &reticle_transform.compute_transform())
            },
            Ability::Fireball => {
                let blueprint = FireballInfo::default();
                blueprint.fire(&mut commands, &caster_transform.compute_transform())
            },
            _ => { 
                let blueprint = DefaultAbilityInfo::default();
                blueprint.fire(&mut commands, &caster_transform.compute_transform())
            },
        };

        // Apply general components
        commands.entity(spawned).insert((
            TEAM_NEUTRAL,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
            TargetsInArea::default(),
            TargetsToEffect::default(),
            TargetsHit::new(1),
            EffectApplyType::OnEnter(OnEnterEffect{
                target_penetration: 2,
                ticks: Ticks::Unlimited { interval: 500 },
                ..default()
            }),
            Homing(player_e),
        ));

        // Apply special components
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

fn show_respawn_ui(
    mut death_timer: Query<&mut Visibility, With<RespawnHolder>>,
    mut death_events: EventReader<DeathEvent>,
    mut spawn_events: EventReader<SpawnEvent>,
    local_player: Res<Player>,
){
    let Ok(mut vis) = death_timer.get_single_mut() else { return };
    for event in spawn_events.iter(){
        if event.player == *local_player{
            *vis = Visibility::Hidden;
        }
    }
    for event in death_events.iter(){        
        if event.player == *local_player{
            *vis = Visibility::Visible;
        }
    }
}

fn tick_respawn_ui(
    mut death_timer: Query<&mut Text, With<RespawnText>>,
    gamemodedetails: ResMut<GameModeDetails>,
    local_player: Res<Player>,
){
    let Ok(mut respawn_text) = death_timer.get_single_mut() else { return };
    if let Some(timer) = gamemodedetails.respawns.get(&*local_player) {
        let new_text = (timer.duration().as_secs() as f32 - timer.elapsed_secs()).floor() as u64;
        respawn_text.sections[1].value = new_text.to_string();
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
    ui: Query<Entity, With<PlayerUI>>,
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

pub struct CastEvent {
    pub caster: Entity,
    pub ability: Ability,
}


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Component, Reflect, FromReflect)]
pub struct Team(pub TeamMask);
// Team masks
bitflags::bitflags! {
    #[derive(Reflect, FromReflect, Default)]
    pub struct TeamMask: u32 {
        const ALL = 1 << 0;
        const TEAM_1 = 1 << 1;
        const TEAM_2 = 1 << 2;
        const TEAM_3 = 1 << 3;
        const NEUTRALS = 1 << 4;
    }
}

pub const TEAM_1: Team = Team(TeamMask::from_bits_truncate(TeamMask::TEAM_1.bits() | TeamMask::ALL.bits()));
pub const TEAM_2: Team = Team(TeamMask::from_bits_truncate(TeamMask::TEAM_2.bits() | TeamMask::ALL.bits()));
pub const TEAM_3: Team = Team(TeamMask::from_bits_truncate(TeamMask::TEAM_3.bits() | TeamMask::ALL.bits()));
pub const TEAM_NEUTRAL: Team = Team(TeamMask::from_bits_truncate(TeamMask::NEUTRALS.bits() | TeamMask::ALL.bits()));
pub const TEAM_ALL: Team = Team(TeamMask::from_bits_truncate(TeamMask::ALL.bits()));



// Collision Grouping Flags
bitflags::bitflags! {
    pub struct Groups: u32 {
        const PLAYER = 1 << 0;
        const TERRAIN = 1 << 1;
        const ABILITY = 1 << 2;
        const GROUND = 1 << 3;
        const FLUFF = 1 << 4;

        const PLAYER_FILTER = Groups::PLAYER.bits() | Groups::TERRAIN.bits()| Groups::GROUND.bits();
        const TERRAIN_FILTER = Groups::PLAYER.bits() ;
        // Make this interact with Terrain and other Abilities when we want cool interactions or no pen walls
        const ABILITY_FILTER = Groups::PLAYER.bits();
        const GROUND_FILTER = Groups::GROUND.bits() | Groups::PLAYER.bits();
        const CAMERA_FILTER = Groups::GROUND.bits();
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
pub const GROUND_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::GROUND.bits()), 
    Group::from_bits_truncate(Groups::GROUND_FILTER.bits())
);
pub const CAMERA_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::GROUND.bits()), 
    Group::from_bits_truncate(Groups::CAMERA_FILTER.bits())
);