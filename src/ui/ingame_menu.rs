use bevy::prelude::*;

use crate::{
    assets::Fonts,
    camera::PlayerCam,
    prelude::InGameSet,
    ui::{hud_editor::EditingHUD, mouse::OpenMenus, ui_bundles::*, ButtonAction},
};

pub struct InGameMenuPlugin;
impl Plugin for InGameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (show_ingame_menu, add_ingame_menu).in_set(InGameSet::Update),
        );
    }
}

fn add_ingame_menu(mut commands: Commands, fonts: Res<Fonts>, query: Query<Entity, Added<PlayerCam>>) {
    let Ok(cam) = query.get_single() else { return };
    commands
        .spawn((ingame_menu(), TargetCamera(cam)))
        .with_children(|parent| {
            parent
                .spawn(ingame_menu_button())
                .insert(ButtonAction::Resume)
                .with_children(|parent| {
                    parent.spawn(ingame_menu_button_text("Resume", &fonts));
                });
            parent.spawn(ingame_menu_button()).with_children(|parent| {
                parent.spawn(ingame_menu_button_text("Pause", &fonts));
            });
            parent.spawn(ingame_menu_button()).with_children(|parent| {
                parent.spawn(ingame_menu_button_text("Settings", &fonts));
            });
            parent
                .spawn(ingame_menu_button())
                .insert(ButtonAction::EditUi)
                .with_children(|parent| {
                    parent.spawn(ingame_menu_button_text("Hud Editor", &fonts));
                });
            parent
                .spawn(ingame_menu_button())
                .insert(ButtonAction::Lobby)
                .with_children(|parent| {
                    parent.spawn(ingame_menu_button_text("Return to Lobby", &fonts));
                });
            parent
                .spawn(ingame_menu_button())
                .insert(ButtonAction::Exit)
                .with_children(|parent| {
                    parent.spawn(ingame_menu_button_text("Exit Game", &fonts));
                });
        });
}

fn show_ingame_menu(
    kb: Res<ButtonInput<KeyCode>>,
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
