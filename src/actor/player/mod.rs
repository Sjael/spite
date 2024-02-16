//! Handling of player
//!

use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    ability::Ability,
    actor::{
        bounty::Bounty,
        cast::{AbilityCastSettings, AbilitySlots, Casting, CooldownMap, HoveredAbility, Slot},
        controller::Controller,
        rank::AbilityRanks,
        IncomingDamageLog, OutgoingDamageLog,
    },
    buff::BuffMap,
    camera::Spectatable,
    classes::warrior::Warrior,
    crowd_control::CCMap,
    prelude::*,
    ui::{
        hud::Trackable,
        store::{StoreBuffer, StoreHistory},
    },
    GameState,
};

pub mod input;

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
        app.register_type::<Players>();
        app.register_type::<LocalPlayer>();

        app.add_event::<SpawnPlayerEvent>();

        app.init_resource::<Players>();

        app.add_plugins(input::InputPlugin);

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
                ActorType::Player(player.clone()),
                player, // Player
                Name::new(format!("Player {}", spawning_id.to_string())),
                ActorState::Alive,
            ))
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
                Warrior,
            ))
            .insert({
                let mut attrs = Attributes::default();
                attrs
                    .set(Stat::Health, 33.0)
                    .set_base(Stat::Speed, 6.0)
                    .set_base(Stat::CharacterResourceRegen, 0.0)
                    .set(Stat::CharacterResource, 0.0)
                    .set(Stat::Gold, 20_000.0);
                attrs
            })
            .insert((
                AbilitySlots::new()
                    .with(Slot::Primary, Ability::BasicAttack)
                    .with(Slot::Slot1, Ability::Frostbolt)
                    .with(Slot::Slot2, Ability::Fireball)
                    .with(Slot::Slot3, Ability::Bomb),
                AbilityCastSettings::default(),
                AbilityRanks::default(),
            ))
            .insert((
                TEAM_1,
                IncomingDamageLog::default(),
                OutgoingDamageLog::default(),
                Bounty::default(),
                CooldownMap::default(),
                CCMap::default(),
                BuffMap::default(),
                Spectatable,
                Casting::default(),
                PlayerInput::default(),
                HoveredAbility::default(),
            ))
            //.insert(NavMeshAffector)
            .id();

        let player_is_owned = event.player_id == **local_player_id;
        if player_is_owned {
            commands.entity(player_entity).insert((Trackable,));
        }
    }
}

/*
pub fn respawn_player(
    mut players: Query<(&mut Visibility, &mut ActorState)>,
    local_player: Res<LocalPlayer>,
) {
    for event in respawn_events.read() {
        let Ok((mut vis, mut state)) = players.get_mut(event.entity) else {
            continue
        };
        *vis = Visibility::Visible;
        *state = ActorState::Alive;
        if event.actor == ActorType::Player(*local_player) {
            // TODO: Set the local camera to focus the local player entity again.
        }
    }
}
*/

pub fn spawn_local_player(mut spawn_events: EventWriter<SpawnPlayerEvent>, local_player_id: Res<LocalPlayerId>) {
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

#[derive(Resource, Deref, Reflect)]
#[reflect(Resource)]
pub struct LocalPlayer(Entity);

impl Default for LocalPlayer {
    fn default() -> Self {
        Self(Entity::from_raw(1))
    }
}

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
    local_player: Option<Res<LocalPlayer>>,
    mut cache: ResMut<Players>,
    truth: Query<(&Player, Entity)>,
) {
    cache.clear();

    for (player, entity) in &truth {
        cache.set(*player, entity);
    }

    if let Some(player_entity) = cache.get(&local_player_id.0) {
        if local_player.is_none() {
            println!("updating player");
            commands.insert_resource(LocalPlayer(player_entity));
        }
    } else {
        if local_player.is_some() {
            println!("removing player");
            commands.remove_resource::<LocalPlayer>();
        }
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
    Component, Reflect, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash, Deref, DerefMut,
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
