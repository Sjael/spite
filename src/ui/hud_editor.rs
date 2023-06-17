use bevy::prelude::*;

use super::ui_bundles::{Draggable, EditableUI, editing_ui_handle, EditingUIHandle};


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

pub struct EditUIEvent;

pub fn give_editable_ui(
    mut commands: Commands,
    draggables: Query<(Entity, Option<&Name>), With<EditableUI>>,
){
    for (entity, name) in draggables.iter(){
        commands.entity(entity).insert(Draggable);
        commands.entity(entity).with_children(|parent|{
            parent.spawn(editing_ui_handle());
        });
    }
}

pub fn remove_editable_ui(
    mut commands: Commands,
    draggables: Query<Entity, (With<EditableUI>, With<Draggable>)>,
    edit_handles: Query<Entity, With<EditingUIHandle>>,
){
    for entity in draggables.iter(){
        commands.entity(entity).remove::<Draggable>();
    }
    for handle in edit_handles.iter(){
        commands.entity(handle).despawn_recursive();
    }
}