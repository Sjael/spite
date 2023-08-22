
use bevy::prelude::*;

use crate::assets::Items;
use crate::{actor::stats::Stat::*, item::Item};
use crate::actor::stats::Stat;

use super::ui_bundles::{ItemAttributes, Category, StoreList, StoreItem, ItemParents, ItemTree, item_image, ItemPriceText};

pub const CATEGORIES: [Stat; 7] = [
    Health,
    CharacterResourceRegen,
    CharacterResourceMax,
    PhysicalPower,
    MagicalPower,
    MagicalPenetration,
    PhysicalPenetration,
];


#[derive(Resource, Default)]
pub struct CategorySorted(pub Vec<Stat>);

#[derive(Resource, Default)]
pub struct ItemInspected(pub Option<Item>);



pub fn sort_items(
    list_query: Query<(Entity, &Children), With<StoreList>>,
    mut item_query: Query<(&mut Style, &ItemAttributes)>,
    mut categories_toggled: ResMut<CategorySorted>,
){
    if categories_toggled.is_changed(){
        for (mut style, item_attributes) in item_query.iter_mut(){
            style.display = Display::default();
            if categories_toggled.0.is_empty() { continue }
            if !item_attributes.0.iter().any(|item|categories_toggled.0.contains(item)){
                style.display = Display::None;
            }
        }
    }
}

pub fn click_category(
    mut interaction_query: Query<
        (&Category, &Interaction),
        (Changed<Interaction>, With<Button>),
    >,
    mut categories_toggled: ResMut<CategorySorted>,
) {
    for (category, interaction) in &mut interaction_query {
        if *interaction != Interaction::Pressed {continue};
        if categories_toggled.0.contains(&category.0) {
            categories_toggled.0.retain(|x| x != &category.0);
        } else {
            categories_toggled.0.push(category.0.clone());
        }
    }
}

pub fn inspect_item(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Item, &Interaction),
        (Changed<Interaction>, With<StoreItem>),
    >,
    mut item_inspected: ResMut<ItemInspected>,
    tree_holder: Query<(Entity, Option<&Children>), With<ItemTree>>,
    parents_holder: Query<(Entity, Option<&Children>), With<ItemParents>>,
    mut price_text: Query<&mut Text, With<ItemPriceText>>,
    items: Res<Items>,
) {
    for (item, interaction) in &mut interaction_query {
        if *interaction != Interaction::Pressed {continue};
        if item_inspected.0 == Some(item.clone()){ continue }; // already inspecting item

        let Ok((tree_entity, tree_children)) = tree_holder.get_single() else{ return };
        item_inspected.0 = Some(item.clone());
        if let Some(children) = tree_children {
            for child in children.iter(){
                commands.entity(*child).despawn_recursive();
            }
        }
        spawn_tree(&mut commands, item.clone(), &items, tree_entity);

        let Ok((parents_entity, parents_children)) = parents_holder.get_single() else{ return };
        if let Some(children) = parents_children {
            for child in children.iter(){
                commands.entity(*child).despawn_recursive();
            }
        }
        commands.entity(parents_entity).with_children(|parent| {
            parent.spawn(item_image(&items, item.clone()));
        });

        let Ok(mut price_text) = price_text.get_single_mut() else{ return };
        price_text.sections[0].value = item.calculate_cost().to_string();
    }
}

fn spawn_tree(commands: &mut Commands, item: Item, item_images: &Res<Items>, parent: Entity) {
    commands.entity(parent).with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
        parent.spawn(item_image(&item_images, item.clone()));
    });
    let parts = item.get_parts();
    for part in parts{
        spawn_tree(commands, part, item_images, parent);
    }
}