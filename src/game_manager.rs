use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    ability::{
        bundles::Caster, Ability, MaxTargetsHit, TargetsHittable, TargetsInArea, TickBehavior,
    },
    actor::{
        cast_ability,
        player::{Player, PlayerEntity},
        stats::{Attributes, Stat},
        view::Reticle,
        AbilityRanks, IncomingDamageLog, RespawnEvent,
    },
    area::homing::Homing,
    inventory::Inventory,
    prelude::*,
    ui::ui_bundles::{PlayerUI, RespawnHolder, RespawnText},
    GameState,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InGameSet {
    Pre,
    Update,
    Post,
}
pub struct GameManagerPlugin;
impl Plugin for GameManagerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Bounty>();

        app.insert_resource(GameModeDetails::default());
        app.insert_resource(Player::new(1507)); // change to be whatever the server says
        app.insert_resource(PlayerEntity(None));
        app.insert_resource(TeamRoster::default());
        app.insert_resource(Scoreboard::default());

        app.add_event::<DeathEvent>();
        app.add_event::<AbilityFireEvent>();
        app.add_event::<FireHomingEvent>();

        app.configure_sets(
            Update,
            InGameSet::Update.run_if(in_state(GameState::InGame)),
        );
        app.configure_sets(
            PreUpdate,
            InGameSet::Pre.run_if(in_state(GameState::InGame)),
        );
        app.configure_sets(
            PostUpdate,
            InGameSet::Pre.run_if(in_state(GameState::InGame)),
        );

        app.add_systems(First, check_deaths.run_if(in_state(GameState::InGame)));
        app.add_systems(
            Update,
            (
                spool_gold,
                increment_bounty,
                handle_respawning,
                show_respawn_ui,
                tick_respawn_ui,
                place_ability.after(cast_ability),
                place_homing_ability,
            )
                .in_set(InGameSet::Update),
        );
        app.add_systems(Last, despawn_dead.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Resource)]
pub struct TeamRoster {
    pub teams: HashMap<Team, Vec<Player>>,
}
impl Default for TeamRoster {
    fn default() -> Self {
        let team1 = vec![Player::new(1507), Player::new(404)];
        let team2 = vec![Player::new(420), Player::new(1)];

        let inner = HashMap::from([(TEAM_1, team1), (TEAM_2, team2)]);
        Self { teams: inner }
    }
}

#[derive(Default)]
pub enum GameMode {
    #[default]
    Arena,
    Tutorial,
    Conquest,
    Practice,
}

#[derive(Resource)]
pub struct GameModeDetails {
    pub mode: GameMode,
    pub start_timer: i32,
    pub respawns: HashMap<Entity, Respawn>,
    pub spawn_points: HashMap<ActorType, Transform>,
}

pub struct Respawn {
    actortype: ActorType,
    timer: Timer,
}

pub enum Spawnpoint {
    RedBuff,
    BlueBuff,
    Chaos,
    Order,
}

impl Default for GameModeDetails {
    fn default() -> Self {
        Self {
            // Pre-game timer
            start_timer: -65,
            respawns: HashMap::new(),
            mode: GameMode::default(),
            spawn_points: HashMap::new(),
        }
    }
}

#[derive(Component)]
pub struct Fountain;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Bounty {
    pub xp: f32,
    pub gold: f32,
}

impl Default for Bounty {
    fn default() -> Self {
        Self {
            xp: 200.0,
            gold: 250.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Scoreboard(pub HashMap<Player, PlayerInfo>);

#[derive(Default)]
pub struct PlayerInfo {
    pub kda: KDA,
    pub inv: Inventory,
    pub logs: LoggedNumbers,
    // account_name: String,
    // account_icon: Image,
    // ping: u32,
    // class: GameClass,
}

pub enum GameClass {
    Rogue,
    Warrior,
    Mage,
    Shaman,
    Cultist,
}

#[derive(Default)]
pub struct KDA {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

#[derive(Default)]
pub struct LoggedNumbers {
    pub gold_acquired: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub damage_mitigated: u32,
    pub healing_dealt: u32,
}

fn place_homing_ability(
    mut commands: Commands,
    mut cast_events: EventReader<FireHomingEvent>,
    caster: Query<(&GlobalTransform, &Team)>,
) {
    for event in cast_events.read() {
        let Ok((caster_transform, team)) = caster.get(event.caster) else {
            return;
        };

        let spawned = event
            .ability
            .get_bundle(&mut commands, &caster_transform.compute_transform());

        // Apply general components
        commands.entity(spawned).insert((
            Name::new("Tower shot"),
            team.clone(),
            Homing(event.target),
            Caster(event.caster),
        ));

        // if has a shape
        commands.entity(spawned).insert((
            TargetsInArea::default(),
            TargetsHittable::default(),
            MaxTargetsHit::new(1),
            TickBehavior::individual(),
        ));
    }
}

fn place_ability(
    mut commands: Commands,
    mut cast_events: EventReader<AbilityFireEvent>,
    caster: Query<(&GlobalTransform, &Team, &AbilityRanks)>,
    reticle: Query<&GlobalTransform, With<Reticle>>,
    procmaps: Query<&ProcMap>,
) {
    let Ok(reticle_transform) = reticle.get_single() else {
        return;
    };
    for event in cast_events.read() {
        let Ok((caster_transform, team, _ranks)) = caster.get(event.caster) else {
            return;
        };

        // Get ability-specific components
        let spawned;

        if event.ability.on_reticle() {
            spawned = event
                .ability
                .get_bundle(&mut commands, &reticle_transform.compute_transform());
        } else {
            spawned = event
                .ability
                .get_bundle(&mut commands, &caster_transform.compute_transform());
        }

        // Apply general components
        commands.entity(spawned).insert((
            //Name::new("ability #tick number"),
            team.clone(),
            Caster(event.caster),
        ));

        //let rank = ranks.map.get(&event.ability).cloned().unwrap_or_default();
        //let scaling = rank.current as u32 * event.ability.get_scaling();

        // Apply special proc components
        if let Ok(procmap) = procmaps.get(event.caster) {
            if let Some(behaviors) = procmap.0.get(&event.ability) {
                for behavior in behaviors {
                    match behavior {
                        AbilityBehavior::Homing => (),
                        AbilityBehavior::OnHit => (),
                    }
                }
            }
        }
    }
}

fn handle_respawning(
    time: Res<Time>,
    mut gamemodedetails: ResMut<GameModeDetails>,
    mut respawn_events: EventWriter<RespawnEvent>,
    local_player: Res<Player>,
) {
    //let theredeemed = commands.spawn(()).id();
    // need to combine the loading and respawn system to go from an event
    // need character enum probably

    gamemodedetails.respawns.retain(|redeemed, respawn| {
        respawn.timer.tick(time.delta());
        if respawn.timer.finished() {
            respawn_events.send(RespawnEvent {
                entity: redeemed.clone(),
                actor: respawn.actortype.clone(),
            });
            if respawn.actortype == ActorType::Player(*local_player) {

            }
        }
        !respawn.timer.finished()
    });
}

fn show_respawn_ui(
    mut death_timer: Query<&mut Visibility, With<RespawnHolder>>,
    mut death_events: EventReader<DeathEvent>,
    mut spawn_events: EventReader<RespawnEvent>,
    local_player: Res<Player>,
) {
    let Ok(mut vis) = death_timer.get_single_mut() else {
        return;
    };
    for event in spawn_events.read() {
        if event.actor == ActorType::Player(*local_player) {
            *vis = Visibility::Hidden;
        }
    }
    for event in death_events.read() {
        if event.actor == ActorType::Player(*local_player) {
            *vis = Visibility::Visible;
        }
    }
}

fn tick_respawn_ui(
    mut death_timer: Query<&mut Text, With<RespawnText>>,
    gamemodedetails: ResMut<GameModeDetails>,
    local_entity: Res<PlayerEntity>,
) {
    let Ok(mut respawn_text) = death_timer.get_single_mut() else {
        return;
    };
    let Some(local) = local_entity.0 else { return };
    let Some(respawn) = gamemodedetails.respawns.get(&local) else {
        return;
    };
    let new_text =
        (respawn.timer.duration().as_secs() as f32 - respawn.timer.elapsed_secs()).floor() as u64;
    respawn_text.sections[1].value = new_text.to_string();
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
    pub actor: ActorType,
    pub killers: Vec<Entity>,
}

fn check_deaths(
    the_damned: Query<
        (Entity, &IncomingDamageLog, &ActorType, &Attributes),
        Changed<IncomingDamageLog>,
    >,
    mut death_events: EventWriter<DeathEvent>,
) {
    const TIME_FOR_KILL_CREDIT: u64 = 30;
    for (guy, damagelog, actortype, attributes) in the_damned.iter() {
        if attributes.get(Stat::Health) > 0.0 {
            continue;
        }

        let mut killers = Vec::new();
        for instance in damagelog.list.iter().rev() {
            if Instant::now().duration_since(instance.when)
                > Duration::from_secs(TIME_FOR_KILL_CREDIT)
            {
                break;
            }
            //let Ok(attacker) = the_guilty.get(instance.attacker) else {continue};
            killers.push(instance.attacker);
        }

        death_events.send(DeathEvent {
            entity: guy,
            actor: actortype.clone(),
            killers,
        });
    }
}

fn despawn_dead(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    mut the_damned: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut ActorState,
            Option<&Bounty>,
        ),
        With<ActorType>,
    >,
    mut attributes: Query<(&mut Attributes, &ActorType)>,
    mut gamemodedetails: ResMut<GameModeDetails>,
    ui: Query<Entity, With<PlayerUI>>,
    local_player: Res<Player>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for event in death_events.read() {
        let mut is_dead_player = false;
        match event.actor {
            ActorType::Player(player) => {
                if player == *local_player {
                    let Ok(ui) = ui.get_single() else { continue };
                    commands.entity(ui).despawn_recursive(); // simply spectate something else in new ui system
                }
                let dead_guy = scoreboard.0.entry(player).or_default();
                dead_guy.kda.deaths += 1;
                is_dead_player = true;
            }
            _ => (),
        }
        let respawn_timer = 8; // change to calculate based on level and game time, or static for jg camps

        let Ok((mut transform, mut vis, mut state, bounty)) = the_damned.get_mut(event.entity)
        else {
            return;
        };

        for (index, awardee) in event.killers.iter().enumerate() {
            let Ok((mut attributes, awardee_actor)) = attributes.get_mut(*awardee) else {
                continue;
            };

            if let Some(bounty) = bounty {
                let gold = attributes.get_mut(Stat::Gold);
                *gold += bounty.gold;
                let xp = attributes.get_mut(Stat::Xp);
                *xp += bounty.xp;
            }

            if !is_dead_player {
                continue;
            }
            if let ActorType::Player(killer) = awardee_actor {
                let killer_scoreboard = scoreboard.0.entry(*killer).or_default();
                if index == 0 {
                    killer_scoreboard.kda.kills += 1;
                } else {
                    killer_scoreboard.kda.assists += 1;
                }
            }
        }

        //commands.entity(event.entity).despawn_recursive();
        *state = ActorState::Dead;
        *transform = Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        };
        *vis = Visibility::Hidden;
        gamemodedetails.respawns.insert(
            event.entity.clone(),
            Respawn {
                actortype: event.actor.clone(),
                timer: Timer::new(Duration::from_secs(respawn_timer), TimerMode::Once),
            },
        );
    }
}

// Make only increment when damageable? Aegis before guarenteed death is meta
// now LMAO
fn increment_bounty(mut the_notorious: Query<&mut Bounty>, time: Res<Time>) {
    for mut wanted in the_notorious.iter_mut() {
        wanted.gold += 2.0 * time.delta_seconds();
        wanted.xp += 4.0 * time.delta_seconds();
    }
}

fn spool_gold(mut attribute_query: Query<&mut Attributes, With<Player>>, time: Res<Time>) {
    let gold_per_second = 3.0;
    for mut attributes in attribute_query.iter_mut() {
        let gold = attributes.get_mut(Stat::Gold);
        *gold += gold_per_second * time.delta_seconds();
    }
}

#[derive(Event)]
pub struct AbilityFireEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Event)]
pub struct FireHomingEvent {
    pub caster: Entity,
    pub ability: Ability,
    pub target: Entity,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub struct Team(pub TeamMask);
// Team masks
bitflags::bitflags! {
    #[derive(Reflect, Default)]
    pub struct TeamMask: u32 {
        const ALL = 1 << 0;
        const TEAM_1 = 1 << 1;
        const TEAM_2 = 1 << 2;
        const TEAM_3 = 1 << 3;
        const NEUTRALS = 1 << 4;
    }
}

pub const TEAM_1: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_1.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_2: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_2.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_3: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_3.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_NEUTRAL: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::NEUTRALS.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_ALL: Team = Team(TeamMask::from_bits_truncate(TeamMask::ALL.bits()));

#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    Ability,
    Ground,
    Wall,
    Fluff,
}

// Collision Grouping Flags
pub const PLAYER_LAYER: CollisionLayers =
    //CollisionLayers::new([Layer::Player], [Layer::Player, Layer::Wall, Layer::Ground]);
    CollisionLayers::from_bits(Layer::Player as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);

pub const GROUND_LAYER: CollisionLayers =
    //CollisionLayers::new([Layer::Ground], [Layer::Player, Layer::Wall, Layer::Ground]);
    CollisionLayers::from_bits(Layer::Player as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);
pub const WALL_LAYER: CollisionLayers =
    //CollisionLayers::new([Layer::Wall], [Layer::Player, Layer::Wall, Layer::Ground]);
    CollisionLayers::from_bits(Layer::Player as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);
pub const ABILITY_LAYER: CollisionLayers =
/*CollisionLayers::new(
    [Layer::Ability],
    [Layer::Player, Layer::Wall, Layer::Ground, Layer::Ability],
);*/
    CollisionLayers::from_bits(Layer::Player as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);

#[derive(Component)]
pub struct ProcMap(HashMap<Ability, Vec<AbilityBehavior>>);

pub enum AbilityBehavior {
    Homing,
    OnHit,
}
