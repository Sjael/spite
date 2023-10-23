use std::io::Cursor;

use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use bevy_fly_camera::{camera_movement_system, mouse_motion_system};
use sacred_aurora::prelude::*;
use winit::window::Icon;

fn main() {
    let mut app = App::new();
    app.add_plugins(sacred_aurora::GamePlugin);
    app.add_plugins(sacred_aurora::map::arena::ArenaPlugin);
    // Systems
    app.add_systems(Startup, set_window_icon);
    app.add_systems(
        Update,
        (camera_movement_system, mouse_motion_system)
            .in_set(InGameSet::Update)
            .run_if(in_state(ActorState::Dead)),
    );
    app.run();
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(primary_entity) = primary_window.get_single() else {
        return;
    };
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!("../assets/icons/fireball.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
