use std::time::Duration;

use bevy::prelude::*;
use bevy_debug_texture::DebugTexturePlugin;
use bevy_editor_pls::prelude::*;
use bevy_tweening::TweeningPlugin;
use bevy_xpbd_3d::{prelude::*, PhysicsSchedule};
use inventory::InventoryPlugin;

use ability::{AbilityPlugin, shape::load_ability_shape};
use actor::ActorPlugin;
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
pub mod physics;
pub mod ui;

pub mod prelude {
    pub use crate::{
        ability::stats::{AttributeTag, Attributes, Modifier, Stat},
        actor::{
            ActorState, ActorType,
        },
        area::*,
        assets::{Icons, Models, Scenes},
        game_manager::{InGameSet, TEAM_1, TEAM_2, TEAM_3, TEAM_NEUTRAL},
        inventory::Inventory,
        item::Item,
        physics::*,
    };
    pub use bevy::prelude::*;
    pub use bevy_xpbd_3d::prelude::*;
    pub use oxidized_navigation::NavMeshAffector;
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let default_res = (1200.0, 600.0);

        //Basic
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sacred Aurora".to_string(),
                resolution: default_res.into(),
                ..default()
            }),
            ..default()
        }));

        //Resources + States
        let tick_hz = 64.0;
        app.insert_resource(Time::<Fixed>::from_hz(tick_hz))
            .insert_resource(Time::<Physics>::new_with(Physics::fixed_once_hz(tick_hz)));

        app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
            .add_state::<GameState>();

        app.add_plugins(EditorPlugin::default());
        app.add_plugins(PhysicsPlugins::new(FixedUpdate));
        app.add_plugins(TweeningPlugin);
        app.add_plugins(InventoryPlugin);
        app.add_plugins(DebugTexturePlugin);

        app.add_systems(PostUpdate, crate::debug::physics_mesh::init_physics_meshes);
        app.add_systems(Startup, spawn_spectator_camera);

        app.add_plugins((
            GameAssetPlugin,
            GameManagerPlugin,
            UiPlugin,
            AbilityPlugin,
            ActorPlugin,
            AreaPlugin,
            InputPlugin,
        ))
        .add_systems(PostUpdate, load_ability_shape); // after systems that spawn ability_shape components
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    MainMenu,
}

/// Quick function for getting a duration for tick rates.
pub const fn tick_hz(rate: u64) -> Duration {
    Duration::from_nanos(1_000_000_000 / rate)
}
