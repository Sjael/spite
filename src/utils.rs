use bevy::{
    ecs::system::Commands,
    hierarchy::{Children, DespawnRecursiveExt},
    prelude::*,
    ui::Val,
    window::PrimaryWindow,
};

pub fn despawn_children(commands: &mut Commands, children: Option<&Children>) {
    let Some(children) = children else { return };
    for child in children.iter() {
        commands.entity(*child).despawn_recursive();
    }
}

pub fn floor_tenths(float: f32) -> f32 {
    (float * 10.0).floor() / 10.0
}

pub fn floor_places(float: f32, places: u32) -> f32 {
    let mut deno = 1.0;
    for _ in 0..places {
        deno *= 10.0;
    }
    (float * deno).floor() / deno
}

pub fn get_px(val: Val) -> f32 {
    match val {
        Val::Px(x) => x.floor(),
        _ => 0.0,
    }
}

pub fn window_focused(windows: Query<Option<&Window>, With<PrimaryWindow>>) -> bool {
    match windows
        .get_single()
        .ok()
        .and_then(|windows| windows.map(|window| window.focused))
    {
        Some(focused) => focused,
        _ => false,
    }
}
