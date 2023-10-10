use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    ability::{
        bundles::Caster, Ability, MaxTargetsHit, TargetsHittable, TargetsInArea, TickBehavior,
    },
    actor::{
        cast_ability,
        player::{Player, PlayerEntity},
        stats::{Attributes, Stat},
        view::Reticle,
        AbilityRanks, IncomingDamageLog, InitSpawnEvent, RespawnEvent,
    },
    area::homing::Homing,
    ui::{
        inventory::Inventory,
        ui_bundles::{PlayerUI, RespawnHolder, RespawnText},
    },
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
        app.insert_resource(GameModeDetails::default());
        app.insert_resource(Player::new(1507)); // change to be whatever the server says
        app.insert_resource(PlayerEntity(None));
        app.insert_resource(TeamRoster::default());
        app.insert_resource(Scoreboard::default());

        app.register_type::<Bounty>();
        app.add_state::<CharacterState>();

        app.add_event::<DeathEvent>();
        app.add_event::<AbilityFireEvent>();
        app.add_event::<FireHomingEvent>();

        app.configure_set(
            Update,
            InGameSet::Update.run_if(in_state(GameState::InGame)),
        );
        app.configure_set(
            PreUpdate,
            InGameSet::Pre.run_if(in_state(GameState::InGame)),
        );
        app.configure_set(
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

#[derive(Component, Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CharacterState {
    Alive,
    #[default]
    Dead,
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

// how long things take to respawn
pub struct RespawnTimeDefaults {
    camps: HashMap<ActorType, u32>,
    player_base: f32,
    player_scale_level: f32,
    player_scale_minutes: f32,
}

impl Default for RespawnTimeDefaults {
    fn default() -> Self {
        let camps = HashMap::from([(ActorType::RedBuff, 240), (ActorType::BlueBuff, 120)]);
        Self {
            camps,
            player_base: 3.0,
            player_scale_level: 2.0,
            player_scale_minutes: 0.75,
        }
    }
}

pub enum Spawnpoint {
    RedBuff,
    BlueBuff,
    Chaos,
    Order,
}

impl Spawnpoint {
    fn transform(&self) -> Transform {
        match self {
            Spawnpoint::RedBuff => Transform {
                translation: Vec3::new(0.0, 0.5, -6.0),
                ..default()
            },
            Spawnpoint::BlueBuff => Transform {
                translation: Vec3::new(0.0, 0.5, 6.0),
                ..default()
            },
            Spawnpoint::Chaos => Transform {
                translation: Vec3::new(-10.0, 0.5, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + std::f32::consts::PI),
                ..default()
            },
            Spawnpoint::Order => Transform {
                translation: Vec3::new(10.0, 0.5, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                ..default()
            },
        }
    }
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
    stats: Attributes,
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
    for event in cast_events.iter() {
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
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
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
    for event in cast_events.iter() {
        let Ok((caster_transform, team, ranks)) = caster.get(event.caster) else {
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

        let rank = ranks.map.get(&event.ability).cloned().unwrap_or_default();
        let scaling = rank.current as u32 * event.ability.get_scaling();

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
    mut spawn_events: EventWriter<InitSpawnEvent>,
    mut respawn_events: EventWriter<RespawnEvent>,
    mut character_state_next: ResMut<NextState<CharacterState>>,
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
                character_state_next.set(CharacterState::Alive);
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
    for event in spawn_events.iter() {
        if event.actor == ActorType::Player(*local_player) {
            *vis = Visibility::Hidden;
        }
    }
    for event in death_events.iter() {
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

pub struct ActorInfo {
    pub entity: Entity,
    pub actor: ActorType,
}

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub enum ActorType {
    BlueBuff,
    RedBuff,
    Player(Player),
}

fn check_deaths(
    the_damned: Query<
        (Entity, &IncomingDamageLog, &ActorType, &Attributes),
        Changed<IncomingDamageLog>,
    >,
    the_guilty: Query<&ActorType>,
    mut death_events: EventWriter<DeathEvent>,
) {
    const TIME_FOR_KILL_CREDIT: u64 = 30;
    for (guy, damagelog, actortype, attributes) in the_damned.iter() {
        let hp = attributes.get(&Stat::Health.as_tag()).unwrap_or(&1.0);
        if *hp > 0.0 {
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
            &mut CharacterState,
            Option<&Bounty>,
        ),
        With<ActorType>,
    >,
    mut attributes: Query<(&mut Attributes, &ActorType)>,
    mut gamemodedetails: ResMut<GameModeDetails>,
    ui: Query<Entity, With<PlayerUI>>,
    mut character_state_next: ResMut<NextState<CharacterState>>,
    local_player: Res<Player>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for event in death_events.iter() {
        let mut is_dead_player = false;
        match event.actor {
            ActorType::Player(player) => {
                if player == *local_player {
                    character_state_next.set(CharacterState::Dead);
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
                let gold = attributes.entry(Stat::Gold.into()).or_default();
                *gold += bounty.gold;
                let xp = attributes.entry(Stat::Xp.into()).or_default();
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
        *state = CharacterState::Dead;
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

// Make only increment when damageable? Aegis before guarenteed death is meta now LMAO
fn increment_bounty(mut the_notorious: Query<&mut Bounty>, time: Res<Time>) {
    for mut wanted in the_notorious.iter_mut() {
        wanted.gold += 2.0 * time.delta_seconds();
        wanted.xp += 4.0 * time.delta_seconds();
    }
}

fn spool_gold(mut attribute_query: Query<&mut Attributes, With<Player>>, time: Res<Time>) {
    let gold_per_second = 3.0;
    for mut attributes in attribute_query.iter_mut() {
        let gold = attributes.entry(Stat::Gold.as_tag()).or_insert(2700.0);
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
    Group::from_bits_truncate(Groups::PLAYER_FILTER.bits()),
);
pub const TERRAIN_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::TERRAIN.bits()),
    Group::from_bits_truncate(Groups::TERRAIN_FILTER.bits()),
);
pub const ABILITY_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::ABILITY.bits()),
    Group::from_bits_truncate(Groups::ABILITY_FILTER.bits()),
);
pub const GROUND_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::GROUND.bits()),
    Group::from_bits_truncate(Groups::GROUND_FILTER.bits()),
);
pub const CAMERA_GROUPING: CollisionGroups = CollisionGroups::new(
    Group::from_bits_truncate(Groups::GROUND.bits()),
    Group::from_bits_truncate(Groups::CAMERA_FILTER.bits()),
);

#[derive(Component)]
pub struct ProcMap(HashMap<Ability, Vec<AbilityBehavior>>);

pub enum AbilityBehavior {
    Homing,
    OnHit,
}
