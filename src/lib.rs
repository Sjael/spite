use std::time::Duration;

use ability::{shape::load_ability_shape, AbilityPlugin};
use actor::ActorPlugin;
use area::AreaPlugin;
use assets::GameAssetPlugin;
use bevy::prelude::*;
use bevy_debug_texture::DebugTexturePlugin;
use bevy_editor_pls::prelude::*;
use bevy_tweening::TweeningPlugin;
use bevy_xpbd_3d::prelude::*;
use buff::BuffPlugin;
use camera::{spawn_spectator_camera, CameraPlugin};
use crowd_control::CCPlugin;
use inventory::InventoryPlugin;
use stats::StatsPlugin;
use ui::UiPlugin;

use crate::{
    classes::{hunter::HunterPlugin, warrior::WarriorPlugin},
    session::SessionPlugin,
};

pub mod ability;
pub mod actor;
pub mod area;
pub mod assets;
pub mod buff;
pub mod camera;
pub mod classes;
pub mod crowd_control;
pub mod debug;
pub mod inventory;
pub mod item;
pub mod map;
pub mod physics;
pub mod previous;
pub mod session;
pub mod stats;
pub mod ui;
pub mod utils;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_xpbd_3d::prelude::*;
    pub use oxidized_navigation::NavMeshAffector;

    pub use crate::{
        actor::{ActorState, ActorType},
        area::*,
        assets::{Icons, Models, Scenes},
        inventory::Inventory,
        item::Item,
        physics::*,
        previous::*,
        session::{director::InGameSet, team::*},
        stats::{AttributeTag, Attributes, Modifier, Stat},
    };
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
            .insert_resource(Time::<Physics>::new_with(Physics::fixed_once_hz(tick_hz)))
            .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)));

        app.add_state::<GameState>();

        app.add_plugins((
            EditorPlugin::default(),
            PhysicsPlugins::new(FixedUpdate),
            TweeningPlugin,
            DebugTexturePlugin,
        ));

        app.add_plugins((
            SessionPlugin,
            GameAssetPlugin,
            CameraPlugin,
            InventoryPlugin,
            UiPlugin,
            AbilityPlugin,
            ActorPlugin,
            AreaPlugin,
            BuffPlugin,
            StatsPlugin,
            CCPlugin,
            WarriorPlugin,
            HunterPlugin,
        ));

        app.add_systems(Startup, spawn_spectator_camera);
        app.add_systems(
            PostUpdate,
            (
                crate::debug::physics_mesh::init_physics_meshes,
                load_ability_shape, // after systems that spawn ability_shape components
            ),
        );
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
