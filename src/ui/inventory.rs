
use bevy::prelude::*;

use crate::assets::Items;
use crate::game_manager::InGameSet;
use crate::item::ITEM_TOTALS;
use crate::{actor::stats::Stat::*, item::Item};
use crate::actor::stats::Stat;

use super::ButtonAction;
use super::styles::{PRESSED_BUTTON, NORMAL_BUTTON};
use super::ui_bundles::*;

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

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CategorySorted::default());
        app.insert_resource(ItemInspected::default());
        
        app.add_systems(Update, (
            sort_items,
            click_category,
            inspect_item,
            clear_filter,
        ).in_set(InGameSet::Update));
    }
}

// TODO
// HIDE ONLY ITEMS IN LIST node

pub fn sort_items(
    list_query: Query<&Children, With<StoreList>>,
    mut item_query: Query<(&mut Style, &ItemAttributes)>,
    categories_toggled: Res<CategorySorted>,
){
    if categories_toggled.is_changed(){
        let Ok(children) = list_query.get_single() else {return};
        for child in children.iter(){
            let Ok((mut style, item_attributes)) = item_query.get_mut(*child) else {continue};
            style.display = Display::default();
            if categories_toggled.0.is_empty() { continue }
            if !item_attributes.0.iter().any(|item|categories_toggled.0.contains(item)){
                style.display = Display::None;
            }
        }
    }
}


pub fn click_category(
    interaction_query: Query<
        (&Category, &Interaction, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_query: Query<&mut BackgroundColor, With<Category>>,
    mut categories_toggled: ResMut<CategorySorted>,
) {
    for (category, interaction, entity) in &interaction_query {
        if *interaction != Interaction::Pressed {continue};
        for mut color in &mut button_query{
            *color = NORMAL_BUTTON.into();
        }
        let Ok(mut color) = button_query.get_mut(entity) else {continue};
        if !categories_toggled.0.contains(&category.0) {
            categories_toggled.0 = vec![category.0.clone()];
            *color = PRESSED_BUTTON.into();
        } else {
            categories_toggled.0 = Vec::new();
        }
        return
    }
}



pub fn clear_filter(
    mut interaction_query: Query<(&ButtonAction, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut category_query: Query<(&mut ButtonToggle, &mut BackgroundColor), With<Category>>,
    mut categories_toggled: ResMut<CategorySorted>,
){    
    for (button_action, interaction) in &mut interaction_query {
        if *button_action != ButtonAction::ClearFilter { continue }
        if *interaction != Interaction::Pressed {continue}
        categories_toggled.0 = Vec::new();
        for (mut toggle, mut color) in category_query.iter_mut(){
            *toggle = ButtonToggle::Off;
            *color = NORMAL_BUTTON.into();
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
    mut name_text: Query<&mut Text, (With<ItemNameText>, Without<ItemPriceText>)>,
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
        let tree = spawn_tree(&mut commands, item.clone(), &items, tree_entity);
        commands.entity(tree_entity).push_children(&[tree]);

        let Ok((parents_entity, parents_children)) = parents_holder.get_single() else{ return };
        if let Some(children) = parents_children {
            for child in children.iter(){
                commands.entity(*child).despawn_recursive();
            }
        }
        commands.entity(parents_entity).with_children(|parent| {
            let supers = item.get_ancestors();
            for i in supers{
                parent.spawn(item_image(&items, i));
            }
        });

        let Ok(mut price_text) = price_text.get_single_mut() else{ return };
        //price_text.sections[0].value = item.calculate_price().to_string();
        price_text.sections[0].value = ITEM_TOTALS.get(&item).cloned().unwrap_or_default().price.to_string();
        let Ok(mut name_text) = name_text.get_single_mut() else{ return };
        name_text.sections[0].value = item.to_string();
    }
}

fn spawn_tree(commands: &mut Commands, item: Item, item_images: &Res<Items>, parent: Entity) -> Entity {
   
    let vert = commands.spawn(vert()).id();
    let hori = commands.spawn(hori()).id();
    let image = commands.spawn(item_image(item_images, item.clone())).id();
    commands.entity(vert).push_children(&[image, hori]);
    let parts = item.get_parts();
    for part in parts{
        let sub = spawn_tree(commands, part, item_images, parent);
        commands.entity(hori).push_children(&[sub]);
    }
    vert

}