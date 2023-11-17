use bevy::{prelude::*, utils::HashMap};

use crate::{
    actor::player::{Player, PlayerEntity},
    game_manager::{Respawn, Team},
    prelude::{ActorType, TEAM_1, TEAM_2},
};
// Game director
//
// Controller of the match over time.

pub struct DirectorPlugin;
impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameModeDetails::default());
        app.insert_resource(Player::new(1507)); // change to be whatever the server says
        app.insert_resource(PlayerEntity(None));
        app.insert_resource(TeamRoster::default());
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

#[derive(Resource)]
pub struct GameModeDetails {
    pub mode: GameMode,
    pub start_timer: i32,
    pub respawns: HashMap<Entity, Respawn>,
    pub spawn_points: HashMap<ActorType, Transform>,
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
