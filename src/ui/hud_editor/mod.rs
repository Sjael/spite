use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::{
    assets::Fonts,
    prelude::InGameSet,
    ui::{
        holding::Reposition,
        hud_editor::save::{Layout, Offset},
        ui_bundles::{
            button, button_text, editing_ui_buttons, editing_ui_handle, editing_ui_label, EditableUI, EditingUIHandle,
            UiForEditingUi,
        },
        ButtonAction, RootUI,
    },
    utils::{floor_places, get_px},
};

pub struct HudEditorPlugin;
impl Plugin for HudEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EditingHUD>();
        app.register_type::<Layout>();
        app.add_event::<HudEditEvent>();

        app.add_systems(Startup, read_hud_layout);

        app.add_systems(Update, apply_layout.in_set(InGameSet::Update));

        app.add_systems(OnEnter(EditingHUD::Yes), give_editable_ui);
        app.add_systems(Update, scale_ui.run_if(in_state(EditingHUD::Yes)));
        app.add_systems(Update, reset_or_save_ui);
        app.add_systems(OnExit(EditingHUD::Yes), remove_editable_ui);
    }
}

fn read_hud_layout(mut commands: Commands) {
    commands.insert_resource(Layout::read_file());
}

fn apply_layout(
    mut query: Query<(&mut Style, &mut Transform, &EditableUI), Added<EditableUI>>,
    layout: Option<Res<Layout>>,
) {
    let Some(layout) = layout else { return };
    for (mut style, mut transform, edit_type) in query.iter_mut() {
        let key = edit_type.get_name();
        if layout.0.contains_key(&key) {
            let Some(custom) = layout.0.get(&key) else { continue };
            style.left = Val::Px(custom.left);
            style.top = Val::Px(custom.top);
            transform.scale = Vec3::splat(custom.scale);
        }
    }
}

fn give_editable_ui(
    mut commands: Commands,
    mut editables: Query<(Entity, &Parent, &mut Visibility), With<EditableUI>>,
    root: Query<Entity, With<RootUI>>,
    names: Query<&Name>,
    fonts: Res<Fonts>,
) {
    for (entity, parent_entity, mut vis) in editables.iter_mut() {
        if *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
            commands.entity(entity).insert(WasHidden);
        }
        commands.entity(entity).insert(Reposition::default());
        commands.entity(entity).with_children(|parent| {
            parent.spawn(editing_ui_handle()).with_children(|parent| {
                if let Ok(name) = names.get(parent_entity.get()) {
                    parent.spawn(editing_ui_label(name, &fonts));
                }
            });
        });
    }

    let Ok(root_entity) = root.get_single() else { return };
    commands
        .spawn(editing_ui_buttons())
        .with_children(|parent| {
            parent
                .spawn(button())
                .insert(ButtonAction::ResetUi)
                .with_children(|parent| {
                    parent.spawn(button_text("Reset", &fonts));
                });
            parent
                .spawn(button())
                .insert(ButtonAction::SaveUi)
                .with_children(|parent| {
                    parent.spawn(button_text("Save", &fonts));
                });
        })
        .set_parent(root_entity);
}

fn scale_ui(
    mut editables: Query<&mut Transform, With<EditableUI>>,
    edit_handles: Query<(&Parent, &Interaction), With<EditingUIHandle>>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        for (parent, interaction) in edit_handles.iter() {
            if interaction != &Interaction::Hovered {
                continue
            }
            let Ok(mut transform) = editables.get_mut(parent.get()) else { continue };
            if event.y > 0.0 {
                transform.scale += Vec3::splat(0.05);
            } else {
                transform.scale += Vec3::splat(-0.05);
            }
        }
    }
}

fn reset_or_save_ui(
    mut commands: Commands,
    mut editables: Query<(&mut Style, &mut Transform, &EditableUI), With<EditableUI>>,
    mut hud_edit_events: EventReader<HudEditEvent>,
) {
    let Some(event) = hud_edit_events.read().next() else { return };
    if *event == HudEditEvent::Reset {
        for (mut style, mut transform, _) in editables.iter_mut() {
            style.top = Val::Px(0.0);
            style.left = Val::Px(0.0);
            transform.scale = Vec3::ONE;
        }
    } else if *event == HudEditEvent::Save {
        let mut layout = Layout::default();
        for (style, transform, edit_type) in editables.iter_mut() {
            // currently saving even the unmoved ui, might change later but checking if float is 0.0 is annoying
            let name = edit_type.get_name();
            let top = get_px(style.top);
            let left = get_px(style.left);
            let scale = floor_places(transform.scale.x, 2);
            let offset = Offset { top, left, scale };
            layout.0.insert(name, offset);
        }
        commands.insert_resource(layout.clone());
        layout.save();
    }
}

fn remove_editable_ui(
    mut commands: Commands,
    mut editables: Query<(Entity, &mut Visibility, Option<&WasHidden>), (With<EditableUI>, With<Reposition>)>,
    editing_ui: Query<Entity, With<UiForEditingUi>>,
    edit_handles: Query<Entity, With<EditingUIHandle>>,
) {
    for (entity, mut vis, was_hidden) in editables.iter_mut() {
        commands.entity(entity).remove::<Reposition>();
        if was_hidden.is_some() {
            commands.entity(entity).remove::<WasHidden>();
            *vis = Visibility::Hidden;
        }
    }
    for handle in edit_handles.iter() {
        commands.entity(handle).despawn_recursive();
    }
    if let Ok(editing_ui_entity) = editing_ui.get_single() {
        commands.entity(editing_ui_entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct WasHidden;

#[derive(Event, PartialEq)]
pub enum HudEditEvent {
    Reset,
    Save,
}

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

pub mod save;
