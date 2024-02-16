use bevy::prelude::*;

use crate::{
    assets::Fonts,
    ui::{hud_editor::EditingHUD, mouse::OpenMenus, ui_bundles::*, ButtonAction},
    GameState,
};

pub struct InGameMenuPlugin;
impl Plugin for InGameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), add_ingame_menu);
        app.add_systems(Update, show_ingame_menu);
    }
}

fn add_ingame_menu(mut commands: Commands, fonts: Res<Fonts>) {
    commands.spawn(ingame_menu()).with_children(|parent| {
        parent
            .spawn(ingame_menu_button())
            .insert(ButtonAction::Resume)
            .with_children(|parent| {
                parent.spawn(ingame_menu_button_text("Resume".to_string(), &fonts));
            });
        parent.spawn(ingame_menu_button()).with_children(|parent| {
            parent.spawn(ingame_menu_button_text("Pause".to_string(), &fonts));
        });
        parent.spawn(ingame_menu_button()).with_children(|parent| {
            parent.spawn(ingame_menu_button_text("Settings".to_string(), &fonts));
        });
        parent
            .spawn(ingame_menu_button())
            .insert(ButtonAction::EditUi)
            .with_children(|parent| {
                parent.spawn(ingame_menu_button_text("Hud Editor".to_string(), &fonts));
            });
        parent
            .spawn(ingame_menu_button())
            .insert(ButtonAction::Lobby)
            .with_children(|parent| {
                parent.spawn(ingame_menu_button_text(
                    "Return to Lobby".to_string(),
                    &fonts,
                ));
            });
        parent
            .spawn(ingame_menu_button())
            .insert(ButtonAction::Exit)
            .with_children(|parent| {
                parent.spawn(ingame_menu_button_text("Exit Game".to_string(), &fonts));
            });
    });
}

fn show_ingame_menu(
    kb: Res<Input<KeyCode>>,
    menu_state: Res<State<OpenMenus>>,
    mut next_state: ResMut<NextState<OpenMenus>>,
    editing_hud: Res<State<EditingHUD>>,
    mut editing_hud_next: ResMut<NextState<EditingHUD>>,
) {
    if kb.just_pressed(KeyCode::Escape) {
        // check if editing hud first
        if *editing_hud == EditingHUD::Yes {
            editing_hud_next.set(EditingHUD::No);
            // TODO make escape rollback changes to as they were, in case of accidentally editing hud
            return
        }
        next_state.set(menu_state.toggle_ingamemenu());
    }
}
