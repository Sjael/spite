use bevy::{prelude::*, window::PrimaryWindow};
use bevy_editor_pls::editor::Editor;

use crate::{prelude::InGameSet, ui::hud_editor::EditingHUD, GameState};

pub fn build_mouse(app: &mut App) {
    app.init_state::<MouseState>();
    app.init_state::<OpenMenus>();

    app.configure_sets(
        Update,
        FreeMouseSet
            .run_if(in_state(MouseState::Free))
            .run_if(in_state(GameState::InGame)),
    );

    app.add_systems(
        Update,
        (menu_show, menu_toggle, mouse_toggle, mouse_show).in_set(InGameSet::Update),
    );
}

fn mouse_toggle(
    kb: Res<ButtonInput<KeyCode>>,
    menu_state: Res<State<OpenMenus>>,
    editing_hud: Res<State<EditingHUD>>,
    mut next_state: ResMut<NextState<MouseState>>,
) {
    if menu_state.0.is_empty() && *editing_hud == EditingHUD::No {
        next_state.set(MouseState::Locked);
    }

    if kb.pressed(KeyCode::Space) || !menu_state.0.is_empty() || *editing_hud == EditingHUD::Yes {
        next_state.set(MouseState::Free);
    }
}

fn mouse_show(
    mouse_state: Res<State<MouseState>>,
    editor: Option<Res<Editor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !mouse_state.is_changed() {
        return
    }
    let editor_active = editor.map(|state| state.active()).unwrap_or(false);
    let Ok(window_is_focused) = windows.get_single().and_then(|window| Ok(window.focused)) else { return };
    let Ok(mut window) = windows.get_single_mut() else { return };
    if *mouse_state == MouseState::Locked && window_is_focused && !editor_active {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn menu_toggle(
    kb: Res<ButtonInput<KeyCode>>,
    menu_state: Res<State<OpenMenus>>,
    mut next_state: ResMut<NextState<OpenMenus>>,
) {
    if kb.just_pressed(KeyCode::KeyT) {
        next_state.set(menu_state.toggle(MenuType::DamageLog));
    } else if kb.just_pressed(KeyCode::KeyY) {
        next_state.set(menu_state.toggle(MenuType::DeathRecap));
    } else if kb.just_pressed(KeyCode::KeyK) {
        next_state.set(menu_state.toggle(MenuType::Abilities));
    } else if kb.just_pressed(KeyCode::Tab) {
        next_state.set(menu_state.toggle(MenuType::Scoreboard));
    }
}

fn menu_show(menu_state: Res<State<OpenMenus>>, mut inner_menu_query: Query<(&mut Visibility, &MenuType)>) {
    if !menu_state.is_changed() {
        return
    }

    for (mut tab_vis, menu_type) in &mut inner_menu_query {
        if menu_state.0.contains(menu_type) {
            *tab_vis = Visibility::Visible;
        } else {
            *tab_vis = Visibility::Hidden;
        }
    }
}

#[derive(States, Clone, Default, Debug, Hash, Eq, PartialEq)]
pub struct OpenMenus(pub Vec<MenuType>);

impl OpenMenus {
    pub fn toggle(&self, menu: MenuType) -> OpenMenus {
        if self.0.contains(&menu) {
            Self(Vec::new())
        } else {
            Self(vec![menu])
        }
    }
    pub fn toggle_ingamemenu(&self) -> OpenMenus {
        if !self.0.is_empty() {
            Self(Vec::new())
        } else {
            Self(vec![MenuType::InGameMenu])
        }
    }
}

#[derive(Clone, Component, Debug, Hash, PartialEq, Eq)]
pub enum MenuType {
    Store,
    Scoreboard,
    DamageLog,
    DeathRecap,
    Abilities,
    InGameMenu,
}

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum MouseState {
    Free,
    #[default]
    Locked,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FreeMouseSet;
