use bevy::{prelude::*, input::mouse::MouseWheel, window::PrimaryWindow};

use crate::assets::Fonts;

use super::{ui_bundles::{Draggable, EditableUI, editing_ui_handle, EditingUIHandle, editing_ui_label, editing_ui, button, button_text, UiForEditingUi}, mouse::MouseState, ButtonAction};


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

pub fn give_editable_ui(
    mut commands: Commands,
    mut editables: Query<(Entity, &Parent, &mut Visibility), With<EditableUI>>,
    names: Query<&Name>,
    fonts: Res<Fonts>,
){
    for (entity, parent_entity, mut vis) in editables.iter_mut(){

        if *vis == Visibility::Hidden{            
            *vis = Visibility::Visible;
            commands.entity(entity).insert(WasHidden);
        }
        commands.entity(entity).insert(Draggable::Unbound);
        commands.entity(entity).with_children(|parent|{
            parent.spawn(editing_ui_handle()).with_children(|parent| {
                if let Ok(name) = names.get(parent_entity.get()){
                    parent.spawn(editing_ui_label(name.as_str().to_string(), &fonts));
                }
            });
        });
    }

    commands.spawn(editing_ui()).with_children(|parent| {
        parent.spawn(button()).insert(
            ButtonAction::ResetUi,
        ).with_children(|parent| {
            parent.spawn(button_text("Reset".to_string(),&fonts));
        });
        parent.spawn(button()).insert(
            ButtonAction::EditUi,
        ).with_children(|parent| {
            parent.spawn(button_text("Save".to_string(),&fonts));
        });
    });
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
    let Some(_) = window.cursor_position() else { return };  
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
    editing_ui: Query<Entity, With<UiForEditingUi>>,
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
    if let Ok(editing_ui_entity) = editing_ui.get_single(){
        commands.entity(editing_ui_entity).despawn_recursive();
    }
}

pub struct ResetUiEvent;

pub fn reset_editable_ui(
    mut editables: Query<(&mut Style, &mut Transform), With<EditableUI>>,
    mut reset_ui_events: EventReader<ResetUiEvent>,
){
    let Some(_) = reset_ui_events.iter().next() else {return};
    for (mut style, mut transform) in editables.iter_mut(){
        style.position.top = Val::Px(0.0);
        style.position.left = Val::Px(0.0);
        transform.scale = Vec3::ONE;
    }
}