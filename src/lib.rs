use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

use ability::{shape::load_ability_shape, AbilityPlugin};
use assets::GameAssetPlugin;
use bevy_tweening::TweeningPlugin;
use buff::{BuffPlugin};
use crowd_control::{tick_ccs, CCMap};
use game_manager::GameManagerPlugin;
use input::InputPlugin;
use player::PlayerPlugin;
use stats::StatsPlugin;
use ui::UiPlugin;
use view::ViewPlugin;

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
                watch_for_changes: true,
                ..default()
            }),
    );

    //Resources + States
    app.insert_resource(GameTimer::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_state::<GameState>();

    app.add_plugin(EditorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            enabled: true,
            style: Default::default(),
            //mode: DebugRenderMode::COLLIDER_SHAPES,
            ..default()
        })
        .add_plugin(TweeningPlugin);

    app.add_plugin(GameAssetPlugin)
        .add_plugin(GameManagerPlugin)
        .add_plugin(ViewPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BuffPlugin)
        .add_plugin(StatsPlugin)
        .add_plugin(AbilityPlugin)
        .add_plugin(InputPlugin)
        // move buffs / cc into character plugin at some point
        .add_systems((load_ability_shape, tick_game, buff::tick_buffs, tick_ccs));
    app.register_type::<CCMap>();
    //app.register_type::<BuffMap>();
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
pub mod assets;
pub mod buff;
pub mod crowd_control;
pub mod game_manager;
pub mod input;
pub mod item;
pub mod player;
pub mod stats;
pub mod ui;
pub mod view;
