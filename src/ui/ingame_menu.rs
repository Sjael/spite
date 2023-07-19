use crate::assets::Fonts;

use super::{ui_bundles::*, ButtonAction};
use bevy::prelude::*;


#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash, )]
pub enum InGameMenu{
    Open,
    #[default]
    Closed
}

impl InGameMenu{
    pub fn toggle(&self) -> Self{
        match self{
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

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
        parent.spawn(ingame_menu_button())
        .insert(
            ButtonAction::EditUi
        ).with_children(|parent| {   
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
    mut ingame_menu: Query<&mut Visibility, With<InGameMenuUi>>,
    state: Res<State<InGameMenu>>,
){
    let Ok(mut vis) = ingame_menu.get_single_mut() else {return};
    if *state == InGameMenu::Closed{
        *vis = Visibility::Hidden;
    } else{
        *vis = Visibility::Visible;
    }
}

pub fn state_ingame_menu(
    kb: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<InGameMenu>>,
    state: Res<State<InGameMenu>>,
){
    if kb.just_pressed(KeyCode::Escape){
        next_state.set(state.toggle());
    }
}