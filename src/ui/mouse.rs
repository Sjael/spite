use bevy::{prelude::*, window::{PrimaryWindow}};
use bevy_editor_pls::editor::Editor;

use crate::{ui::ui_bundles::{StoreMain, TabPanel, TabMenuType, TabMenuWrapper}, GameState};

use super::ingame_menu::InGameMenuOpen;


#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum MouseState {
    #[default]
    Free,
    Locked,
}

#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash, )]
pub enum TabMenuOpen{
    Open,
    #[default]
    Closed
}
impl TabMenuOpen{
    pub fn toggle(&self) -> Self{
        match self{
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}


pub fn window_focused(windows: Query<Option<&Window>, With<PrimaryWindow>>) -> bool {
    match windows.get_single().ok().and_then(|windows| windows.map(|window| window.focused)) {
        Some(focused) => focused,
        _ => false,
    }
}

pub fn free_mouse(
    mouse_is_free: Res<State<MouseState>>,
    editor: Option<Res<Editor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
){    
    let editor_active = editor.map(|state| state.active()).unwrap_or(false);
    let Ok(window_is_focused) = windows.get_single().and_then(|window| Ok(window.focused)) else {
        return
    };
    let Ok(mut window) = windows.get_single_mut() else {
        return
    };
    if mouse_is_free.0 == MouseState::Locked && window_is_focused && !editor_active{
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else{
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
    } 
}

pub fn mouse_with_free_key(
    kb: Res<Input<KeyCode>>,
    tab_menu_open: Res<State<TabMenuOpen>>,
    ingame_menu_open: Res<State<InGameMenuOpen>>,
    mut next_state: ResMut<NextState<MouseState>>,
){
    let mouse_key_held = kb.pressed(KeyCode::Space);    
    if tab_menu_open.0 == TabMenuOpen::Closed && ingame_menu_open.0 == InGameMenuOpen::Closed{
        next_state.set(MouseState::Locked);
    }
    if mouse_key_held || tab_menu_open.0 == TabMenuOpen::Open || ingame_menu_open.0 == InGameMenuOpen::Open{
        next_state.set(MouseState::Free);
    }
}


pub struct MenuEvent{
    pub menu: TabMenuType,
}

pub fn menu_toggle(
    kb: Res<Input<KeyCode>>,
    mut store: Query<(&mut Visibility, &ComputedVisibility), (With<StoreMain>, Without<TabPanel>)>,
    mut tab_panel: Query<(&mut Visibility, &ComputedVisibility), With<TabPanel>>,
    mut inner_menu_query: Query<(&mut Visibility, &TabMenuWrapper, &ComputedVisibility), (Without<TabPanel>, Without<StoreMain>)>,
    mut menu_event: EventWriter<MenuEvent>,
    mut next_state: ResMut<NextState<TabMenuOpen>>,
){
    let Ok((mut store_vis, store_computed_vis)) = store.get_single_mut() else { return };
    let Ok((mut panel_vis, panel_computed_vis)) = tab_panel.get_single_mut() else { return };
    if kb.just_pressed(KeyCode::R) {
        if store_computed_vis.is_visible(){
            *store_vis = Visibility::Hidden;
            next_state.set(TabMenuOpen::Closed);
        } else{
            *store_vis = Visibility::Visible;
            next_state.set(TabMenuOpen::Open);
        }
        if panel_computed_vis.is_visible(){            
            *panel_vis = Visibility::Hidden;
            for (mut tab_vis, _, _) in &mut inner_menu_query{
                *tab_vis = Visibility::Hidden;
            }    
        }
    }
    let mut tab_panel_opened = TabMenuType::None;
    if kb.just_pressed(KeyCode::T){
        tab_panel_opened = TabMenuType::DamageLog;
    } else if kb.just_pressed(KeyCode::Y){
        tab_panel_opened = TabMenuType::DeathRecap;
    } else if kb.just_pressed(KeyCode::K){
        tab_panel_opened = TabMenuType::Abilities;
    } else if kb.just_pressed(KeyCode::Tab){
        tab_panel_opened = TabMenuType::Scoreboard;
    }
    if tab_panel_opened != TabMenuType::None{ 
        for (mut tab_vis, tabtype, tab_computed_vis) in &mut inner_menu_query{
            *tab_vis = Visibility::Hidden;
            if tabtype.0 == tab_panel_opened{                
                if tab_computed_vis.is_visible() && panel_computed_vis.is_visible(){
                    *panel_vis = Visibility::Hidden;
                    next_state.set(TabMenuOpen::Closed);
                } else{
                    *tab_vis = Visibility::Visible;
                    *panel_vis = Visibility::Visible;
                    next_state.set(TabMenuOpen::Open);
                }
            }
        }
        
        if store_computed_vis.is_visible(){
            *store_vis = Visibility::Hidden;
        }
    }
    menu_event.send(MenuEvent{
        menu: tab_panel_opened,
    })
}

