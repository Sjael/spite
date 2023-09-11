
use bevy::prelude::*;

use crate::actor::player::Player;
use crate::actor::view::Spectating;
use crate::assets::Items;
use crate::game_manager::InGameSet;
use crate::item::ITEM_TOTALS;
use crate::{actor::stats::Stat::*, item::Item};
use crate::actor::stats::{Stat, Attributes};

use super::{ButtonAction, SpectatingSet};
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

#[derive(Component, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct Inventory(pub [Option<Item>; 6]);

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CategorySorted::default());
        app.insert_resource(ItemInspected::default());
        app.add_event::<BuyItemEvent>();
        app.register_type::<Inventory>();
        
        app.add_systems(Update, (
            sort_items,
            click_category,
            inspect_item,
            clear_filter,
            try_buy_item,
        ).in_set(SpectatingSet));
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
    mut text_set: ParamSet<(
        Query<&mut Text, With<ItemPriceText>>,
        Query<&mut Text, With<ItemDiscountText>>,
        Query<&mut Text, With<ItemNameText>>,
    )>,
    items: Res<Items>,
    player: Res<Spectating>,
    buyers: Query<&Inventory>,
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

        let Ok(inventory) = buyers.get(player.0) else{ return };

        if let Ok(mut price_text) = text_set.p0().get_single_mut() {
            price_text.sections[0].value = item.get_price().to_string();
        }
        if let Ok(mut discount_text) = text_set.p1().get_single_mut() {
            discount_text.sections[0].value = item.calculate_discount(&inventory).to_string();
        }
        if let Ok(mut name_text) = text_set.p2().get_single_mut() {
            name_text.sections[0].value = item.to_string();
        }
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

#[derive(Event)]
pub struct BuyItemPressEvent;
#[derive(Event)]
pub struct BuyItemEvent{
    pub item: Item,
    pub player: Entity,
}

fn try_buy_item(
    mut commands: Commands,
    mut events: EventReader<BuyItemEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    slot_query: Query<(Entity, Option<&Children>, &BuildSlotNumber)>,
    items: Res<Items>,
){
    for event in events.iter(){
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.player) else {continue};
        let gold = attributes.entry(Stat::Gold.as_tag()).or_insert(1.0);
        let discounted_price = event.item.calculate_discount(&inventory) as f32;
        if *gold > discounted_price {
            // remove components
            for item in event.item.get_parts(){
                if let Some(index) = inventory.iter().position(|old| *old == Some(item.clone())){
                    for (slot_e, children, number) in &slot_query{
                        if number.0 != index as u32 + 1 {continue}
                        commands.entity(slot_e).despawn_descendants();
                    }
                    inventory.0[index] = None;
                }
            }
            let open_slot = inventory.0.iter().position(|x| x == &None);
            if let Some(slot) = open_slot{
                // pay the price
                *gold -= discounted_price;
                // insert item
                inventory.0[slot] = Some(event.item.clone());
                for (slot_e, children, number) in &slot_query{
                    if number.0 != slot as u32 + 1 {continue}
                    let new_item = commands.spawn(item_image_build(&items, event.item.clone())).id();
                    commands.entity(new_item).set_parent(slot_e);
                }
            } else {
                dbg!("inventory full");
            }
        }else {
            dbg!("you're too broke for that lol");
        }
    }
}
