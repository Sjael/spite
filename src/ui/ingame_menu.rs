use crate::assets::Fonts;

use super::{ui_bundles::*, mouse::TabMenuOpen, ButtonAction};
use bevy::prelude::*;


pub fn add_ingame_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
) {    
    commands.spawn(ingame_menu()).with_children(|parent| {
        parent.spawn(ingame_menu_button())
        .insert(
            ButtonAction::Resume
        ).with_children(|parent| {            
            parent.spawn(ingame_menu_button_text("Resume".to_string(),&fonts));      
        });      
        parent.spawn(ingame_menu_button()).with_children(|parent| {   
            parent.spawn(ingame_menu_button_text("Pause".to_string(),&fonts));    
        });      
        parent.spawn(ingame_menu_button()).with_children(|parent| {   
            parent.spawn(ingame_menu_button_text("Settings".to_string(),&fonts));    
        });      
        parent.spawn(ingame_menu_button()).with_children(|parent| {   
            parent.spawn(ingame_menu_button_text("Hud Editor".to_string(),&fonts));    
        });      
        parent.spawn(ingame_menu_button())
        .insert(
            ButtonAction::Lobby
        ).with_children(|parent| {   
            parent.spawn(ingame_menu_button_text("Return to Lobby".to_string(),&fonts));    
        });      
        parent.spawn(ingame_menu_button())
        .insert(
            ButtonAction::Exit
        ).with_children(|parent| {   
            parent.spawn(ingame_menu_button_text("Exit Game".to_string(),&fonts));        
        });          
    });
}

pub fn toggle_ingame_menu(
    mut ingame_menu: Query<(&mut Visibility, &ComputedVisibility), With<InGameMenu>>,
    kb: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<TabMenuOpen>>,
){
    let Ok((mut vis, computed_vis)) = ingame_menu.get_single_mut() else {return};
    if kb.just_pressed(KeyCode::Escape){
        if computed_vis.is_visible(){
            *vis = Visibility::Hidden;
            next_state.set(TabMenuOpen::Closed);
        } else{
            *vis = Visibility::Visible;
            next_state.set(TabMenuOpen::Open);
        }
    }
}