use bevy::prelude::*;

use crate::assets::Fonts;

use super::ui_bundles::{Draggable, EditableUI, editing_ui_handle, EditingUIHandle, editing_ui_label};


#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash, )]
pub enum EditingHUD{
    Yes,
    #[default]
    No
}


impl EditingHUD{
    pub fn toggle(&self) -> Self{
        match self{
            Self::Yes => Self::No,
            Self::No => Self::Yes,
        }
    }
}

#[derive(Component)]
pub struct WasHidden;

pub struct EditUIEvent;

pub fn give_editable_ui(
    mut commands: Commands,
    mut editables: Query<(Entity, Option<&Name>, &mut Visibility), With<EditableUI>>,
    fonts: Res<Fonts>,
){
    for (entity, name, mut vis) in editables.iter_mut(){

        if *vis == Visibility::Hidden{            
            *vis = Visibility::Visible;
            commands.entity(entity).insert(WasHidden);
        }
        commands.entity(entity).insert(Draggable);
        commands.entity(entity).with_children(|parent|{
            parent.spawn(editing_ui_handle()).with_children(|parent| {
                if let Some(name) = name{
                    parent.spawn(editing_ui_label(name.as_str().to_string(), &fonts));
                }
            });
        });
    }
}


pub fn remove_editable_ui(
    mut commands: Commands,
    mut editables: Query<(Entity, &mut Visibility, Option<&WasHidden>), (With<EditableUI>, With<Draggable>)>,
    edit_handles: Query<Entity, With<EditingUIHandle>>,
){
    for (entity, mut vis, was_hidden) in editables.iter_mut(){
        if let Some(_) = was_hidden{
            commands.entity(entity).remove::<WasHidden>();
            *vis = Visibility::Hidden;
        }
        commands.entity(entity).remove::<Draggable>();
    }
    for handle in edit_handles.iter(){
        commands.entity(handle).despawn_recursive();
    }
}