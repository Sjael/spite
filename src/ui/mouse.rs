use bevy::{prelude::*, window::PrimaryWindow};
use bevy_editor_pls::editor::Editor;

use crate::ui::ui_bundles::{StoreMain, TabMenuType, TabPanel};

use super::{hud_editor::EditingHUD, ingame_menu::InGameMenu};

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum MouseState {
    Free,
    #[default]
    Locked,
}

#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
pub enum StoreMenu {
    Open,
    #[default]
    Closed,
}

#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
pub enum TabMenu {
    Open,
    #[default]
    Closed,
}
impl TabMenu {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
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

pub fn free_mouse(
    mouse_state: Res<State<MouseState>>,
    editor: Option<Res<Editor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let editor_active = editor.map(|state| state.active()).unwrap_or(false);
    let Ok(window_is_focused) = windows.get_single().and_then(|window| Ok(window.focused)) else {
        return;
    };
    let Ok(mut window) = windows.get_single_mut() else { return };
    if mouse_state.is_changed() {
        if *mouse_state == MouseState::Locked && window_is_focused && !editor_active {
            window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
            window.cursor.visible = false;
        } else {
            window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

pub fn mouse_with_free_key(
    kb: Res<Input<KeyCode>>,
    tab_menu: Res<State<TabMenu>>,
    store_menu: Res<State<StoreMenu>>,
    ingame_menu: Res<State<InGameMenu>>,
    editing_hud: Res<State<EditingHUD>>,
    mut next_state: ResMut<NextState<MouseState>>,
) {
    if *tab_menu == TabMenu::Closed
        && *store_menu == StoreMenu::Closed
        && *ingame_menu == InGameMenu::Closed
        && *editing_hud == EditingHUD::No
    {
        next_state.set(MouseState::Locked);
    }

    if kb.pressed(KeyCode::Space)
        || *tab_menu == TabMenu::Open
        || *store_menu == StoreMenu::Open
        || *ingame_menu == InGameMenu::Open
        || *editing_hud == EditingHUD::Yes
    {
        next_state.set(MouseState::Free);
    }
}

#[derive(Event)]
pub struct MenuEvent {
    pub menu: TabMenuType,
}

pub fn menu_toggle(
    kb: Res<Input<KeyCode>>,
    mut store: Query<&mut Visibility, With<StoreMain>>,
    mut tab_panel: Query<&mut Visibility, (With<TabPanel>, Without<StoreMain>)>,
    mut inner_menu_query: Query<
        (&mut Visibility, &TabMenuType, &ComputedVisibility),
        (Without<TabPanel>, Without<StoreMain>),
    >,
    mut next_tab_state: ResMut<NextState<TabMenu>>,
    mut next_store_state: ResMut<NextState<StoreMenu>>,
) {
    let Ok(mut store_vis) = store.get_single_mut() else { return };
    let Ok(mut panel_vis) = tab_panel.get_single_mut() else { return };
    use Visibility::*;
    if kb.just_pressed(KeyCode::R) {
        if *panel_vis == Visible {
            *panel_vis = Hidden;
            next_tab_state.set(TabMenu::Closed);
        }
        if *store_vis == Hidden {
            *store_vis = Visible;
            next_store_state.set(StoreMenu::Open);
        } else {
            *store_vis = Hidden;
            next_store_state.set(StoreMenu::Closed);
        }
    }
    let mut tab_panel_opened = TabMenuType::None;
    if kb.just_pressed(KeyCode::T) {
        tab_panel_opened = TabMenuType::DamageLog;
    } else if kb.just_pressed(KeyCode::Y) {
        tab_panel_opened = TabMenuType::DeathRecap;
    } else if kb.just_pressed(KeyCode::K) {
        tab_panel_opened = TabMenuType::Abilities;
    } else if kb.just_pressed(KeyCode::Tab) {
        tab_panel_opened = TabMenuType::Scoreboard;
    }
    if tab_panel_opened != TabMenuType::None {
        if *store_vis == Visible {
            *store_vis = Hidden;
            next_store_state.set(StoreMenu::Closed);
        }
        for (mut tab_vis, tabtype, computed_tab_vis) in &mut inner_menu_query {
            if *tabtype == tab_panel_opened {
                if computed_tab_vis.is_visible() && *panel_vis == Visible {
                    *panel_vis = Hidden;
                    next_tab_state.set(TabMenu::Closed);
                } else {
                    *tab_vis = Inherited;
                    *panel_vis = Visible;
                    next_tab_state.set(TabMenu::Open);
                }
            } else {
                *tab_vis = Hidden;
            }
        }
    }
}
