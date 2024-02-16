use bevy::{prelude::*, window::PrimaryWindow};

use crate::{prelude::InGameSet, ui::mouse::MouseState};

pub struct HoldingPlugin;
impl Plugin for HoldingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorHolding(None));

        app.add_systems(
            Update,
            (
                hover_holdable,
                drag_holdable,
                hover_drop_slots.run_if(is_holding),
                drop_holdable.run_if(is_holding),
            )
                .chain()
                .run_if(in_state(MouseState::Free))
                .in_set(InGameSet::Update),
        );
    }
}

fn is_holding(holding: Res<CursorHolding>) -> bool {
    holding.0.is_some()
}

fn hover_holdable(
    handle_query: Query<(Entity, &Interaction, &Parent), (With<DragHandle>, Changed<Interaction>)>,
    mut draggables: Query<(&mut BackgroundColor, &HoverHoldStyle), With<Reposition>>,
    holding: Res<CursorHolding>,
) {
    // when a drag handle changes interaction, color its draggable to show it is interactable
    for (handle_entity, interaction, handle_parent) in &handle_query {
        // check both self and parent entity for draggable components (aka reposition)
        let one_of = vec![handle_entity, handle_parent.get()];
        let mut result = draggables.iter_many_mut(one_of);
        let Some((mut bg, _)) = result.fetch_next() else { continue };

        // only give hover opacity if mouse is over and not holding anything
        // TODO change to have an effect based on HoverHoldStyle
        if holding.0.is_none() && *interaction == Interaction::Hovered {
            *bg = Color::rgba(1.0, 1.0, 1.0, 0.6).into();
        } else {
            *bg = Color::WHITE.into();
        }
    }
}

fn drag_holdable(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    // split these queries because the handle isn't necesaarily what is being dragged, aka the store
    handle_query: Query<(Entity, &Interaction, &Parent), With<DragHandle>>,
    mut draggables: Query<(
        &Parent,
        &Node,
        &GlobalTransform,
        &mut ZIndex,
        &mut Reposition,
        Entity,
    )>,
    parents: Query<(&Node, &GlobalTransform)>,
    mouse: Res<Input<MouseButton>>,
    mut holding: ResMut<CursorHolding>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    if !mouse.pressed(MouseButton::Left) {
        return
    }
    for (handle_entity, interaction, handle_parent) in &handle_query {
        if *interaction != Interaction::Pressed {
            continue
        }
        // check both self and parent entity for draggable components (aka reposition)
        let one_of = vec![handle_entity, handle_parent.get()];
        let mut result = draggables.iter_many_mut(one_of);
        let Some((parent, node, gt, mut z, mut repos, ent)) = result.fetch_next() else { continue };

        if mouse.just_pressed(MouseButton::Left) {
            let Ok((parent_node, parent_gt)) = parents.get(parent.get()) else { continue };
            // holding this entity now
            holding.0 = Some(ent);
            // how far the parent top left is from 0,0
            // globaltransform is the nodes center point btw, thats why we need to gt - size / 2
            repos.parent_offset = parent_gt.translation().xy() - parent_node.size() / 2.0;
            // how far the mouse is from the draggable's top left
            repos.inner_offset = cursor_pos - (gt.translation().xy() - node.size() / 2.0);
            // set the Z to be higher, TODO keep a list with ZTracker what to increment to
            if handle_entity == ent {
                *z = ZIndex::Global(7);
            }
        }
        // new position accounting for parent top-left and mouse inside element top-left aka
        // how far the draggable top-left should be from parent top-left
        repos.pos = cursor_pos - repos.parent_offset - repos.inner_offset;
        // return or we could drag multiple entities at the same time
        // TODO prob should sort by zindex before getting click
        return
    }
}

fn hover_drop_slots(
    holding: Res<CursorHolding>,
    mut hold_query: Query<&DropType, Without<DropSlot>>,
    mut slot_query: Query<(&Interaction, &mut BackgroundColor, &DropType), With<DropSlot>>,
) {
    let Some(held_entity) = holding.0 else { return };
    let Ok(held_dt) = hold_query.get_mut(held_entity) else { return };

    for (interaction, mut bg, slot_dt) in &mut slot_query {
        if held_dt != slot_dt {
            continue
        }
        // TODO change to have an effect based on HoverDropStyle
        if *interaction == Interaction::None {
            *bg = Color::GREEN.into();
        } else {
            *bg = Color::GOLD.into();
        }
    }
}

pub fn drop_holdable(
    mut commands: Commands,
    mut holding: ResMut<CursorHolding>,
    mouse: Res<Input<MouseButton>>,
    mut hold_query: Query<(&mut Style, &Parent, &mut ZIndex, Option<&DropType>), Without<DropSlot>>,
    mut slot_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            Option<&Children>,
            &DropType,
        ),
        With<DropSlot>,
    >,
) {
    if !mouse.just_released(MouseButton::Left) {
        return
    }
    let Some(held_entity) = holding.0 else { return };
    let Ok((mut style, parent, mut _z, opt_held_dt)) = hold_query.get_mut(held_entity) else { return };

    holding.0 = None;
    let Some(held_dt) = opt_held_dt else { return };
    for (slot_entity, interaction, mut bg, children, slot_dt) in &mut slot_query {
        // skip slots we dont interact with and didnt change when hovering
        if held_dt != slot_dt {
            continue
        }
        // reset all the colors of the slots that were changed
        *bg = Color::GRAY.into();
        if *interaction == Interaction::None {
            continue
        }
        // only get here if this is the slot we hovered AND is same type
        // swap only for builds makes sens rn
        commands.entity(held_entity).set_parent(slot_entity);
        if let Some(children) = children {
            let Some(prev_item) = children.iter().next() else { continue };
            commands.entity(*prev_item).set_parent(parent.get());
        }
    }
    style.left = Val::default();
    style.top = Val::default();
}

#[derive(Resource, Default)]
pub struct CursorHolding(pub Option<Entity>);

#[derive(Component, Debug, Default, Clone)]
pub struct Reposition {
    pub pos: Vec2,
    parent_offset: Vec2,
    pub inner_offset: Vec2,
}

#[derive(Component, Debug)]
pub enum HoverHoldStyle {
    Transparent,
    Outline,
}

#[derive(Component, Debug, PartialEq)]
pub enum DropType {
    BuildItem,
    ScoreboardPos,
}

#[derive(Component)]
pub struct DropSlot;

#[derive(Component, Debug)]
pub struct DragHandle;

#[derive(Resource, Debug, Default)]
pub struct ZTracker(pub u32);

#[derive(Component, Debug)]
pub struct DragPhantom;
