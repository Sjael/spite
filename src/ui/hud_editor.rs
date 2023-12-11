use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::assets::Fonts;

use super::{
    ui_bundles::{
        button, button_text, editing_ui, editing_ui_handle, editing_ui_label, Draggable,
        EditableUI, EditingUIHandle, UiForEditingUi,
    },
    ButtonAction,
};

#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
pub enum EditingHUD {
    Yes,
    #[default]
    No,
}

impl EditingHUD {
    pub fn toggle(&self) -> Self {
        match self {
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
) {
    for (entity, parent_entity, mut vis) in editables.iter_mut() {
        if *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
            commands.entity(entity).insert(WasHidden);
        }
        commands.entity(entity).insert(Draggable::Unbound);
        commands.entity(entity).with_children(|parent| {
            parent.spawn(editing_ui_handle()).with_children(|parent| {
                if let Ok(name) = names.get(parent_entity.get()) {
                    parent.spawn(editing_ui_label(name, &fonts));
                }
            });
        });
    }

    commands.spawn(editing_ui()).with_children(|parent| {
        parent
            .spawn(button())
            .insert(ButtonAction::ResetUi)
            .with_children(|parent| {
                parent.spawn(button_text("Reset", &fonts));
            });
        parent
            .spawn(button())
            .insert(ButtonAction::EditUi)
            .with_children(|parent| {
                parent.spawn(button_text("Save", &fonts));
            });
    });
}

pub fn scale_ui(
    mut editables: Query<&mut Transform, With<EditableUI>>,
    edit_handles: Query<(&Parent, &Interaction), With<EditingUIHandle>>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        println!("gamer");
        for (parent, interaction) in edit_handles.iter() {
            if interaction != &Interaction::Hovered {
                continue;
            }
            let Ok(mut transform) = editables.get_mut(parent.get()) else {
                continue;
            };
            if event.y > 0.0 {
                transform.scale += Vec3::splat(0.05);
            } else {
                transform.scale += Vec3::splat(-0.05);
            }
        }
    }
}

#[derive(Event)]
pub struct ResetUiEvent;

pub fn reset_editable_ui(
    mut editables: Query<(&mut Style, &mut Transform), With<EditableUI>>,
    reset_ui_events: EventReader<ResetUiEvent>,
) {
    if reset_ui_events.is_empty() {
        return;
    }
    for (mut style, mut transform) in editables.iter_mut() {
        style.top = Val::Px(0.0);
        style.left = Val::Px(0.0);
        transform.scale = Vec3::ONE;
    }
}

pub fn remove_editable_ui(
    mut commands: Commands,
    mut editables: Query<
        (Entity, &mut Visibility, Option<&WasHidden>),
        (With<EditableUI>, With<Draggable>),
    >,
    editing_ui: Query<Entity, With<UiForEditingUi>>,
    edit_handles: Query<Entity, With<EditingUIHandle>>,
) {
    for (entity, mut vis, was_hidden) in editables.iter_mut() {
        if let Some(_) = was_hidden {
            commands.entity(entity).remove::<WasHidden>();
            *vis = Visibility::Hidden;
        }
        commands.entity(entity).remove::<Draggable>();
    }
    for handle in edit_handles.iter() {
        commands.entity(handle).despawn_recursive();
    }
    if let Ok(editing_ui_entity) = editing_ui.get_single() {
        commands.entity(editing_ui_entity).despawn_recursive();
    }
}
