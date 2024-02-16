//! Game director
//!
//! Controller of the match over time.

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    actor::player::{LocalPlayerId, Player, SpawnPlayerEvent},
    prelude::*,
    session::{mode::GameMode, team::*},
    GameState,
};
// Game director
//
// Controller of the match over time.

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InGameSet {
    Pre,
    Update,
    Post,
}

pub struct DirectorPlugin;
impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameModeDetails::default())
            .insert_resource(LocalPlayerId::new(Player::new(1507)))
            .insert_resource(Respawns::default())
            .insert_resource(SpawnPoints::default());

        app.register_type::<Respawns>().register_type::<SpawnPoints>();

        app.world.spawn((
            Name::new("Team 1"),
            TEAM_1,
            TeamRoster::new(vec![Player::new(1507), Player::new(404)]),
            TeamRespawn::new(Duration::from_secs(10)),
        ));

        app.world.spawn((
            Name::new("Team 2"),
            TEAM_2,
            TeamRoster::new(vec![Player::new(420), Player::new(1)]),
            TeamRespawn::new(Duration::from_secs(10)),
        ));

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

        app.configure_sets(
            FixedUpdate,
            (InGameSet::Pre, InGameSet::Update, InGameSet::Post)
                .chain()
                .run_if(in_state(GameState::InGame))
                .before(PhysicsSet::Prepare)
                .before(PhysicsSet::Sync),
        );
        app.add_systems(
            PostUpdate,
            (start_respawn_timer, tick_respawn_timer).in_set(InGameSet::Post),
        );
    }
}

fn start_respawn_timer(
    changed_states: Query<(&ActorState, &ActorType), Changed<ActorState>>,
    mut respawns: ResMut<Respawns>,
) {
    let respawn_time = 10.0;
    for (state, actor_type) in changed_states.iter() {
        if state.is_alive() {
            continue
        }
        respawns.map.insert(
            actor_type.clone(),
            Timer::from_seconds(respawn_time, TimerMode::Once),
        );
    }
}

fn tick_respawn_timer(
    mut respawns: ResMut<Respawns>,
    spawn_points: Res<SpawnPoints>,
    time: Res<Time>,
    mut spawn_events: EventWriter<SpawnPlayerEvent>,
) {
    respawns.map.retain(|actor_type, timer| {
        timer.tick(time.delta());
        if timer.finished() {
            info!("spawning local player");
            let player = if let &ActorType::Player(player) = actor_type {
                player
            } else {
                Player::new(1)
            };
            spawn_events.send(SpawnPlayerEvent {
                player_id: player,
                transform: spawn_points.get_spawn(actor_type),
            });
        }
        !timer.finished()
    });
}

#[derive(Resource)]
pub struct GameModeDetails {
    pub mode: GameMode,
    pub start_timer: i32,
}

impl Default for GameModeDetails {
    fn default() -> Self {
        Self {
            // Pre-game timer
            start_timer: -65,
            mode: GameMode::Arena,
        }
    }
}

#[derive(Default, Reflect, Resource)]
#[reflect(Resource)]
pub struct Respawns {
    pub map: HashMap<ActorType, Timer>,
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct SpawnPoints {
    map: HashMap<ActorType, Transform>,
}
impl SpawnPoints {
    pub fn get_spawn(&self, actor: &ActorType) -> Transform {
        *self.map.get(actor).unwrap_or(&Transform::default())
    }
}
impl Default for SpawnPoints {
    fn default() -> Self {
        let map = HashMap::from([(
            ActorType::Player(Player::new(1507)),
            Transform::from_translation(Vec3::new(2.0, 0.5, 3.0)).looking_at(Vec3::ZERO, Vec3::Y),
        )]);
        Self { map }
    }
}
