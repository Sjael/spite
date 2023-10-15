use crate::assets::Fonts;
use bevy::{app::AppExit, prelude::*};

use super::{
    ui_bundles::{button, button_text},
    ButtonAction,
};

pub fn spawn_main_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Percent(10.),
                    row_gap: Val::Px(20.),
                    ..default()
                },
                ..default()
            },
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(button())
                .insert(ButtonAction::Play)
                .with_children(|parent| {
                    parent.spawn(button_text("Play", &fonts));
                });
            parent
                .spawn(button())
                .insert(ButtonAction::Settings)
                .with_children(|parent| {
                    parent.spawn(button_text("Settings", &fonts));
                });
            parent
                .spawn(button())
                .insert(ButtonAction::Exit)
                .with_children(|parent| {
                    parent.spawn(button_text("Exit Game", &fonts));
                });
        });
}

pub fn cleanup(mut commands: Commands, root: Query<Entity, With<MainMenuRoot>>) {
    for entity in root.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct MainMenuRoot;

pub fn exit_game_main_menu(kb: Res<Input<KeyCode>>, mut app_exit_writer: EventWriter<AppExit>) {
    if kb.just_pressed(KeyCode::Escape) {
        app_exit_writer.send(AppExit);
    }
}
