use bevy::prelude::*;

use crate::{ability::AbilityTooltip, assets::Fonts, prelude::Icons};

use super::ui_bundles::{spawn_ability_tooltip, Hoverable, Tooltip};

pub fn move_tooltip(
    mut tooltip: Query<&mut Style, With<Tooltip>>,
    mut move_events: EventReader<CursorMoved>,
) {
    let Ok(mut style) = tooltip.get_single_mut() else {
        return;
    };
    if let Some(cursor_move) = move_events.read().next() {
        style.left = Val::Px(cursor_move.position.x);
        style.bottom = Val::Px(cursor_move.position.y);
    }
}

pub fn hide_tooltip(
    // for when you close a menu when hovering, otherwise tooltip stays
    mut tooltip: Query<&mut Visibility, With<Tooltip>>,
) {
    let Ok(mut vis) = tooltip.get_single_mut() else {
        return;
    };
    *vis = Visibility::Hidden;
}

pub fn load_tooltip(
    mut commands: Commands,
    mut tooltip: Query<(&mut Tooltip, &mut Visibility, Option<&Children>, Entity)>,
    hoverables: Query<(Entity, &AbilityTooltip, &Interaction), With<Hoverable>>,
    icons: Res<Icons>,
    fonts: Res<Fonts>,
) {
    let Ok((mut tooltip, mut vis, children, tooltip_entity)) = tooltip.get_single_mut() else {
        return;
    };
    let mut hovered_info: Option<AbilityTooltip> = None;
    for (hovering_entity, ability_info, interaction) in &hoverables {
        match interaction {
            Interaction::None => {
                if tooltip.0.is_some() {
                    tooltip.0 = None;
                }
            }
            Interaction::Hovered | Interaction::Pressed => {
                if let Some(last_hovered) = tooltip.0 {
                    if last_hovered == hovering_entity {
                        return;
                    }
                }
                tooltip.0 = Some(hovering_entity.clone());
                hovered_info = Some(ability_info.clone());
            }
        }
    }
    match hovered_info {
        Some(info) => {
            if let Some(children) = children {
                for child in children.iter() {
                    commands.entity(*child).despawn_recursive();
                }
            }
            let child = spawn_ability_tooltip(&mut commands, &icons, &fonts, &info.clone());
            commands.entity(tooltip_entity).add_child(child);
            *vis = Visibility::Visible;
        }
        _ => {
            *vis = Visibility::Hidden;
        }
    }
}
