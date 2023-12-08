//! Handling of player
//!

use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    ability::{
        buff::BuffMap,
        cast::{
            AbilityCastSettings, Casting, CooldownMap, HoveredAbility, IncomingDamageLog,
            OutgoingDamageLog, WindupTimer,
        },
        crowd_control::{CCMap, CCType},
        rank::AbilityRanks,
    },
    actor::{controller::Controller, player::camera::Spectatable},
    game_manager::Bounty,
    input::SlotBundle,
    prelude::*,
    ui::{
        store::{StoreBuffer, StoreHistory},
        Trackable,
    },
    GameState,
};

pub mod camera;
pub mod input;
pub mod reticle;

pub use input::PlayerInput;

#[derive(Event)]
pub struct SpawnPlayerEvent {
    pub player_id: Player,
    pub transform: Transform,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>();

        app.add_event::<SpawnPlayerEvent>();

        app.init_resource::<Players>();

        app.add_plugins(input::InputPlugin)
            .add_plugins(camera::CameraPlugin);

        app.add_systems(OnEnter(GameState::InGame), spawn_local_player);
        app.add_systems(
            FixedUpdate,
            (init_player, update_players).in_set(InGameSet::Pre),
        );
    }
}

pub fn init_player(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut spawn_events: EventReader<SpawnPlayerEvent>,
    local_player_id: Res<LocalPlayerId>,
) {
    for event in spawn_events.read() {
        let player = event.player_id;
        let spawning_id = player.id.clone();
        info!("spawning player {}", spawning_id);
        // reset the rotation so you dont spawn looking the other way

        let player_entity = commands
            .spawn(SpatialBundle::from_transform(event.transform.clone()))
            .insert((
                //event.actor.clone(), // ActorType
                player, // Player
                Name::new(format!("Player {}", spawning_id.to_string())),
                ActorState::Alive,
            ))
            /*.insert(ControllerBundle {
                controller: Controller {
                    float: Float {
                        spring: Spring {
                            strength: SpringStrength::AngularFrequency(25.0),
                            damping: 0.8,
                        },
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            */
            .insert(Controller::default())
            .insert((
                // physics
                Collider::capsule(1.0, 0.5),
                RigidBody::Dynamic,
                LockedAxes::ACTOR,
                CollisionLayers::PLAYER,
                Friction {
                    dynamic_coefficient: 0.0,
                    static_coefficient: 0.0,
                    combine_rule: CoefficientCombine::Min,
                },
                Restitution {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombine::Min,
                },
            ))
            .insert((
                // Inventory/store
                Inventory::default(),
                StoreHistory::default(),
                StoreBuffer::default(),
            ))
            .insert({
                let mut attrs = Attributes::default();
                attrs
                    .set(Stat::Health, 33.0)
                    .set_base(Stat::Speed, 6.0)
                    .set(Stat::CharacterResource, 33.0)
                    .set(Stat::Gold, 20_000.0);
                attrs
            })
            .insert((
                TEAM_1,
                AbilityCastSettings::default(),
                AbilityRanks::default(),
                IncomingDamageLog::default(),
                OutgoingDamageLog::default(),
                Bounty::default(),
                CooldownMap::default(),
                CCMap::default(),
                BuffMap::default(),
                Spectatable,
                Casting(None),
                WindupTimer(Timer::default()),
                PlayerInput::default(),
                SlotBundle::new(), // Has all the keybinding -> action logic
                HoveredAbility::default(),
            ))
            //.insert(NavMeshAffector)
            .id();

        let player_is_owned = event.player_id == **local_player_id;
        if player_is_owned {
            commands.insert_resource(PlayerInput::default());
            commands.entity(player_entity).insert((Trackable,));
        }
    }
}

/// What should we do when the player dies?
pub fn kill_player(mut commands: Commands, players: Query<(Entity, &Attributes), With<Player>>) {
    for (player_entity, attrs) in &players {
        if attrs.get(Stat::Health) <= 0.0 {}
    }
}

/*
pub fn respawn_player(
    mut players: Query<(&mut Visibility, &mut ActorState)>,
    local_player: Res<LocalPlayer>,
) {
    for event in respawn_events.read() {
        let Ok((mut vis, mut state)) = players.get_mut(event.entity) else {
            continue;
        };
        *vis = Visibility::Visible;
        *state = ActorState::Alive;
        if event.actor == ActorType::Player(*local_player) {
            // TODO: Set the local camera to focus the local player entity again.
        }
    }
}
*/

pub fn spawn_local_player(
    mut spawn_events: EventWriter<SpawnPlayerEvent>,
    local_player_id: Res<LocalPlayerId>,
) {
    info!("spawning local player");
    spawn_events.send(SpawnPlayerEvent {
        player_id: **local_player_id,
        transform: Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
    });
}

#[derive(Resource, Deref)]
pub struct LocalPlayer(Entity);

impl LocalPlayer {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl PartialEq<Entity> for LocalPlayer {
    fn eq(&self, other: &Entity) -> bool {
        self.0 == *other
    }
}

impl PartialEq<LocalPlayer> for Entity {
    fn eq(&self, other: &LocalPlayer) -> bool {
        *self == other.0
    }
}

/// Convenience map for fetching an entity of a specific player.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Players(HashMap<Player, Entity>);

impl Players {
    pub fn get(&self, player: &Player) -> Option<Entity> {
        self.0.get(player).copied()
    }

    pub fn set(&mut self, player: Player, entity: Entity) {
        self.0.insert(player, entity);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub fn update_players(
    mut commands: Commands,
    local_player_id: Res<LocalPlayerId>,
    mut cache: ResMut<Players>,
    truth: Query<(&Player, Entity)>,
) {
    cache.clear();

    for (player, entity) in &truth {
        cache.set(*player, entity);
    }

    if let Some(player_entity) = cache.get(&local_player_id.0) {
        commands.insert_resource(LocalPlayer(player_entity));
    } else {
        commands.remove_resource::<LocalPlayer>();
    }
}

#[derive(Resource, Deref)]
pub struct LocalPlayerId(Player);

impl LocalPlayerId {
    pub fn new(player: Player) -> Self {
        Self(player)
    }
}

impl PartialEq<Player> for LocalPlayerId {
    fn eq(&self, other: &Player) -> bool {
        self.0 == *other
    }
}

impl PartialEq<LocalPlayerId> for Player {
    fn eq(&self, other: &LocalPlayerId) -> bool {
        *self == other.0
    }
}

#[derive(
    Component,
    Reflect,
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    Hash,
    Deref,
    DerefMut,
)]
#[reflect(Component)]
pub struct Player {
    pub id: u32,
}

impl Player {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}
