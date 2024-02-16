use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    ability::Ability,
    assets::{Fonts, Items},
    buff::{BuffInfo, BuffType},
    item::Item,
    prelude::Icons,
    ui::{
        buff_image, color_text,
        holding::Reposition,
        mouse::{FreeMouseSet, MouseState},
        plain_text, tooltip_bg, tooltip_desc, tooltip_image, tooltip_image_wrap, tooltip_title,
    },
    utils::despawn_children,
};

pub struct TooltipPlugin;
impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (hover_hoverable, load_tooltip, move_tooltip, show_tooltip)
                .chain()
                .in_set(FreeMouseSet),
        );
        app.add_systems(OnExit(MouseState::Free), hide_tooltip);
    }
}

fn hover_hoverable(
    mut tooltip: Query<&mut Tooltip>,
    hoverables: Query<(Entity, &Interaction), (Changed<Interaction>, With<Hoverable>)>,
) {
    let Ok(mut tooltip) = tooltip.get_single_mut() else { return };
    for (hovered, interaction) in &hoverables {
        if *interaction == Interaction::Hovered {
            println!("hovering on {}v{}", hovered.index(), hovered.generation());
            tooltip.0 = Some(hovered);
        } else if tooltip.has_this_entity(hovered) {
            tooltip.0 = None;
        }
    }
}

fn load_tooltip(
    mut commands: Commands,
    mut tooltip: Query<(&Tooltip, &mut Visibility, Option<&Children>, Entity), Changed<Tooltip>>,
    hoverables: Query<&Hoverable>,
    icons: Res<Icons>,
    items: Res<Items>,
    fonts: Res<Fonts>,
) {
    let Ok((tooltip, mut vis, children, tt_ent)) = tooltip.get_single_mut() else { return };
    if let Some(hovered) = tooltip.0 {
        let Ok(hovered_info) = hoverables.get(hovered) else { return };
        despawn_children(&mut commands, children);
        let child = hovered_info.spawn_ui(&mut commands, &icons, &items, &fonts);
        commands.entity(tt_ent).add_child(child);
    } else {
        *vis = Visibility::Hidden;
        despawn_children(&mut commands, children);
    }
}

pub fn move_tooltip(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut tooltip: Query<&mut Reposition, With<Tooltip>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok(mut repos) = tooltip.get_single_mut() else { return };
    let Some(pos) = window.cursor_position() else { return };
    // give it a little perspective when moving
    let center_pull = 0.1;
    let center = Vec2::new(window.width(), window.height()) / 2.0;
    repos.pos = pos.lerp(center, center_pull);
}

pub fn show_tooltip(mut tooltip: Query<&mut Visibility, (With<Tooltip>, Changed<Children>)>) {
    let Ok(mut vis) = tooltip.get_single_mut() else { return };
    *vis = Visibility::Visible;
}

fn hide_tooltip(mut tooltip: Query<&mut Visibility, With<Tooltip>>) {
    let Ok(mut vis) = tooltip.get_single_mut() else { return };
    *vis = Visibility::Hidden;
}

#[derive(Component, Debug, Default)]
pub struct Tooltip(pub Option<Entity>);

impl Tooltip {
    fn has_this_entity(&self, entity: Entity) -> bool {
        if let Some(last_hovered) = self.0 {
            last_hovered == entity
        } else {
            false
        }
    }
}

#[derive(Component, Clone)]
pub enum Hoverable {
    Item(Item),
    Ability(Ability),
    Buff(BuffInfo),
}

impl Hoverable {
    fn spawn_ui(&self, commands: &mut Commands, icons: &Res<Icons>, items: &Res<Items>, fonts: &Res<Fonts>) -> Entity {
        match self {
            Hoverable::Item(item) => {
                let image = item.get_image(&items);
                let info = item.info();
                commands
                    .spawn(tooltip_bg())
                    .with_children(|parent| {
                        parent.spawn(tooltip_image(image, 48));
                        parent.spawn(tooltip_title(item.name(), &fonts));
                        parent.spawn(color_text(info.price.to_string(), 12, &fonts, Color::GOLD));
                        for (stat, amount) in info.stats {
                            let line = format!("+ {} {}", amount, stat);
                            parent.spawn(tooltip_desc(line, &fonts));
                        }
                    })
                    .id()
            }
            Hoverable::Ability(ability) => {
                let image = ability.get_image(&icons);
                commands
                    .spawn(tooltip_bg())
                    .with_children(|parent| {
                        parent.spawn(tooltip_image(image, 64));
                        parent.spawn(plain_text(ability.get_name(), 30, &fonts));
                        parent.spawn(tooltip_desc(ability.get_description(), &fonts));
                        parent.spawn(plain_text(ability.get_cooldown().to_string(), 14, &fonts));
                    })
                    .id()
            }
            Hoverable::Buff(info) => {
                let description = format!(
                    "Gives {} {}. Lasts {} seconds. Max {} stacks.",
                    info.amount, info.stat, info.duration, info.max_stacks
                );
                let is_buff = info.bufftype == BuffType::Buff;
                commands
                    .spawn(tooltip_bg())
                    .with_children(|parent| {
                        parent.spawn(tooltip_image_wrap(32)).with_children(|parent| {
                            parent.spawn(buff_image(info.image.clone(), is_buff));
                        });
                        parent.spawn(plain_text(info.name.clone(), 22, &fonts));
                        parent.spawn(plain_text(description, 16, &fonts));
                    })
                    .id()
            }
        }
    }
}
