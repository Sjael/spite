//! Game director
//!
//! Controller of the match over time.

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

use super::team::*;
use crate::{
    actor::player::{LocalPlayerId, Player},
    prelude::*,
    GameState,
};

use super::mode::GameMode;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InGameSet {
    Pre,
    Update,
    Post,
}

pub struct DirectorPlugin;
impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameModeDetails::default());
        app.insert_resource(LocalPlayerId::new(Player::new(1507)));

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
    }
}

#[derive(Resource)]
pub struct GameModeDetails {
    pub mode: GameMode,
    pub start_timer: i32,
    pub spawn_points: HashMap<ActorType, Transform>,
}

impl Default for GameModeDetails {
    fn default() -> Self {
        Self {
            // Pre-game timer
            start_timer: -65,
            mode: GameMode::default(),
            spawn_points: HashMap::new(),
        }
    }
}
