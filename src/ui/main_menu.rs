use bevy::{app::AppExit, prelude::*};

use crate::{
    assets::Fonts,
    ui::{
        ui_bundles::{button, button_text, main_menu},
        ButtonAction, MainMenuRoot,
    },
    GameState,
};

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
        app.add_systems(
            Update,
            exit_game_main_menu.run_if(in_state(GameState::MainMenu)),
        );
        app.add_systems(OnExit(GameState::MainMenu), cleanup);
    }
}

fn spawn_main_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn(main_menu()).with_children(|parent| {
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

fn cleanup(mut commands: Commands, root: Query<Entity, With<MainMenuRoot>>) {
    for entity in root.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn exit_game_main_menu(kb: Res<ButtonInput<KeyCode>>, mut app_exit_writer: EventWriter<AppExit>) {
    if kb.just_pressed(KeyCode::Escape) {
        app_exit_writer.send(AppExit);
    }
}
