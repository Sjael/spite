use bevy::asset::ChangeWatcher;
use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

use ability::shape::load_ability_shape;
use actor::view::ViewPlugin;
use actor::CharacterPlugin;
use area::AreaPlugin;
use assets::GameAssetPlugin;
use bevy_tweening::TweeningPlugin;
use game_manager::GameManagerPlugin;
use input::InputPlugin;
use ui::UiPlugin;

pub fn app_plugins_both(app: &mut App) {
    let default_res = (1500.0, 600.0);

    //Basic
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sacred Aurora".to_string(),
                    resolution: default_res.into(),
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..default()
            }),
    );

    //Resources + States
    app.insert_resource(GameTimer::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_state::<GameState>();

    app.add_plugins((
        EditorPlugin::default(),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin {
            enabled: true,
            style: Default::default(),
            //mode: DebugRenderMode::COLLIDER_SHAPES,
            ..default()
        },
        TweeningPlugin,
    ));

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

pub mod ability;
pub mod actor;
pub mod area;
pub mod assets;
pub mod game_manager;
pub mod input;
pub mod item;
pub mod ui;
