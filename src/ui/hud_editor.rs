use bevy::{prelude::*, input::mouse::MouseWheel, window::PrimaryWindow};

use crate::assets::Fonts;

use super::{ui_bundles::{Draggable, EditableUI, editing_ui_handle, EditingUIHandle, editing_ui_label}, mouse::MouseState};


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

pub fn scale_ui(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut editables: Query<&mut Transform, With<EditableUI>>,
    edit_handles: Query<(&Parent, &Interaction), With<EditingUIHandle>>,
    mut scroll_events: EventReader<MouseWheel>,
    mouse_is_free: Res<State<MouseState>>,
){
    if mouse_is_free.0 != MouseState::Free { return }
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };  
    for event in scroll_events.iter(){
        for (parent, interaction) in edit_handles.iter(){
            if interaction != &Interaction::Hovered { continue }
            let Ok(mut transform) = editables.get_mut(parent.get()) else { continue };
            if event.y > 0.0 {
                transform.scale += Vec3::splat(0.05);
            } else {
                transform.scale += Vec3::splat(-0.05);
            }
        }
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