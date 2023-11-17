use std::time::Duration;

use bevy::prelude::*;
use bevy_debug_texture::DebugTexturePlugin;
use bevy_editor_pls::prelude::*;
use bevy_xpbd_3d::prelude::*;
use inventory::InventoryPlugin;
use bevy_tweening::TweeningPlugin;

use ability::shape::load_ability_shape;
use actor::{view::ViewPlugin, CharacterPlugin};
use area::AreaPlugin;
use assets::GameAssetPlugin;
use game_manager::GameManagerPlugin;
use input::InputPlugin;
use ui::{spectating::spawn_spectator_camera, UiPlugin};

pub mod ability;
pub mod actor;
pub mod area;
pub mod assets;
pub mod debug;
pub mod game_manager;
pub mod input;
pub mod inventory;
pub mod item;
pub mod map;
pub mod ui;

pub mod prelude {
    pub use crate::{
        //ability::{},
        actor::{
            stats::{AttributeTag, Attributes, Modifier, Stat},
            view::Spectatable,
            ActorState, ActorType,
        },
        area::*,
        assets::{Icons, Models, Scenes},
        game_manager::{
            InGameSet, ABILITY_LAYER, GROUND_LAYER, PLAYER_LAYER, TEAM_1, TEAM_2, TEAM_3,
            TEAM_NEUTRAL, WALL_LAYER,
        },
        inventory::Inventory,
        item::Item,
    };
    pub use bevy::prelude::*;
    pub use bevy_xpbd_3d::prelude::*;
    pub use oxidized_navigation::NavMeshAffector;
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let default_res = (1500.0, 600.0);

        //Basic
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sacred Aurora".to_string(),
                resolution: default_res.into(),
                present_mode: bevy::window::PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }));

        //Resources + States
        app.insert_resource(GameTimer::default())
            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
            .add_state::<GameState>();

        app.add_plugins(EditorPlugin::default());
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(TweeningPlugin);
        app.add_plugins(InventoryPlugin);
        app.add_plugins(DebugTexturePlugin);

        app.add_systems(PostUpdate, crate::debug::physics_mesh::init_physics_meshes);
        app.add_systems(Startup, spawn_spectator_camera);

        app.add_plugins((
            GameAssetPlugin,
            GameManagerPlugin,
            ViewPlugin,
            UiPlugin,
            CharacterPlugin,
            AreaPlugin,
            InputPlugin,
        ))
        .add_systems(PostUpdate, load_ability_shape) // after systems that spawn ability_shape components
        .add_systems(Update, tick_game);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    MainMenu,
}

#[derive(Deref, DerefMut, Debug, Clone, Resource)]
pub struct GameTimer(Timer);

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::new(tick_hz(64), TimerMode::Repeating))
    }
}

/// Quick function for getting a duration for tick rates.
pub const fn tick_hz(rate: u64) -> Duration {
    Duration::from_nanos(1_000_000_000 / rate)
}

pub fn tick_game(time: Res<Time>, mut game_timer: ResMut<GameTimer>) {
    game_timer.tick(time.delta());
}

pub fn on_gametick(game_timer: Res<GameTimer>) -> bool {
    game_timer.just_finished()
}
