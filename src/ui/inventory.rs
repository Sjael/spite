
use bevy::prelude::*;

use crate::ability::TargetsInArea;
use crate::actor::player::{Player, PlayerEntity};
use crate::actor::view::Spectating;
use crate::area::{AreaOverlapEvent, AreaOverlapType};
use crate::assets::Items;
use crate::game_manager::{InGameSet, Fountain, CharacterState};
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

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct Inventory(pub [Option<Item>; 6]);

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CategorySorted::default());
        app.insert_resource(ItemInspected::default());
        app.insert_resource(StoreSnapshot::default());
        app.add_event::<StoreEvent>();
        app.add_event::<UndoPressEvent>();
        app.register_type::<Inventory>();
        
        app.add_systems(Update, (
            sort_items,
            click_category,
            inspect_item,
            try_buy_item,
            try_sell_item,
            try_undo_store,
            confirm_buy,
            update_inventory_ui,
            update_discounts.after(inspect_item),
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
            if !item_attributes.0.iter().any(|stat|categories_toggled.0.contains(stat)){
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

pub fn update_discounts(
    mut query: ParamSet<(
        Query<(&mut Text, &ItemDiscount), Changed<ItemDiscount>>,
        Query<(&mut Text, &ItemDiscount)>,
    )>,
    player_entity: Res<PlayerEntity>,
    mut players: ParamSet<(
        Query<(&Inventory, Entity), Changed<Inventory>>,
        Query<&Inventory>,
    )>,
){
    let Some(local) = player_entity.0 else {return};
    for (inv, entity) in players.p0().iter(){
        if entity != local { continue }
        for (mut text, discounted_item) in query.p1().iter_mut(){
            text.sections[0].value = discounted_item.0.calculate_discount(&inv).to_string();
        }
    }
    for (mut text, discounted_item) in query.p0().iter_mut(){
        let inv = if let Ok(inv) = players.p1().get(local) {
            inv.clone()
        }else {continue};
        text.sections[0].value = discounted_item.0.calculate_discount(&inv).to_string();
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
        Query<&mut ItemDiscount, With<ItemDiscountText>>,
        Query<&mut Text, With<ItemNameText>>,
    )>,
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
        let tree = spawn_tree(&mut commands, item.clone(), &items);
        commands.entity(tree_entity).push_children(&[tree]);

        let Ok((parents_entity, parents_children)) = parents_holder.get_single() else{ return };
        if let Some(children) = parents_children {
            for child in children.iter(){
                commands.entity(*child).despawn_recursive();
            }
        }
        commands.entity(parents_entity).with_children(|parent| {
            let ancestors = item.get_ancestors();
            for ancestor in ancestors{
                parent.spawn(store_item(&items, ancestor));
            }
        });

        if let Ok(mut price_text) = text_set.p0().get_single_mut() {
            price_text.sections[0].value = item.get_price().to_string();
        }
        if let Ok(mut discount) = text_set.p1().get_single_mut() {
            discount.0 = item.clone();
        }
        if let Ok(mut name_text) = text_set.p2().get_single_mut() {
            name_text.sections[0].value = item.to_string();
        }
    }
}

fn spawn_tree(commands: &mut Commands, item: Item, item_images: &Res<Items>) -> Entity {
    let vert = commands.spawn(vert()).id();
    let image = commands.spawn(store_item(item_images, item.clone())).id();
    commands.entity(vert).push_children(&[image]);
    let parts = item.get_parts();
    if !parts.is_empty(){    
        let hori = commands.spawn(hori()).id();
        commands.entity(vert).push_children(&[hori]);
        for part in parts{
            let sub = spawn_tree(commands, part, item_images);
            commands.entity(hori).push_children(&[sub]);
        }
    }
    vert
}


fn try_buy_item(
    mut events: EventReader<StoreEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    mut snapshot: ResMut<StoreSnapshot>,
){
    for event in events.iter(){
        if event.direction != TransactionType::Buy { continue }
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.player) else {continue};
        let gold = attributes.entry(Stat::Gold.as_tag()).or_insert(1.0);
        let discounted_price = event.item.calculate_discount(&inventory) as f32;
        if *gold > discounted_price {
            // remove components
            for item in event.item.get_parts(){
                let Some(index) = inventory.iter().position(|old| *old == Some(item.clone())) else {continue};
                inventory.0[index] = None;
            }
            let open_slot = inventory.0.iter().position(|x| x == &None);
            if let Some(slot) = open_slot{
                // pay the price
                *gold -= discounted_price;
                snapshot.gold -= discounted_price as i32;
                // insert item
                inventory.0[slot] = Some(event.item.clone());
            } else {
                dbg!("inventory full");
            }
        }else {
            dbg!("you're too broke for that lol");
        }
    }
}

fn update_inventory_ui(
    mut commands: Commands,
    query: Query<(&Inventory, Entity), Changed<Inventory>>,
    slot_query: Query<(Entity, &BuildSlotNumber)>,
    items: Res<Items>, 
    local_entity: Res<PlayerEntity>,
){
    let Some(local) = local_entity.0 else { return };
    for (inv, entity) in query.iter(){
        if entity != local { continue }
        for (slot_e, index) in &slot_query{
            commands.entity(slot_e).despawn_descendants();
            let Some(item) = inv.get(index.0 as usize - 1).unwrap_or(&None) else { continue };
            let new_item = commands.spawn(item_image_build(&items, item.clone())).id();
            commands.entity(new_item).set_parent(slot_e);
        }
    }
}


fn try_sell_item(
    mut events: EventReader<StoreEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    mut snapshot: ResMut<StoreSnapshot>,
){
    for event in events.iter(){
        if event.direction != TransactionType::Sell { continue }
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.player) else {continue};
        let gold = attributes.entry(Stat::Gold.as_tag()).or_insert(1.0);
        let refund = event.item.get_price() as f32;
        if let Some(index) = inventory.iter().position(|old| *old == Some(event.item.clone())){
            inventory.0[index] = None;
            let sell_price = (refund * 0.67).floor();
            *gold += sell_price;
            snapshot.gold += sell_price as i32;
        }
    }
}

fn try_undo_store(
    mut events: EventReader<UndoPressEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    mut snapshot: ResMut<StoreSnapshot>,
){
    for event in events.iter(){
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.entity) else { continue };
        let gold = attributes.entry(Stat::Gold.as_tag()).or_insert(1.0);
        *inventory = snapshot.inventory;
        *gold -= snapshot.gold as f32;
        snapshot.gold = 0;
    }
}

fn confirm_buy(
    mut snapshot: ResMut<StoreSnapshot>,
    buyers: Query<&Inventory>,
    local_entity: Res<PlayerEntity>,
    mut area_events: EventReader<AreaOverlapEvent>,
    sensors: Query<&TargetsInArea, With<Fountain>>,
){
    let Some(event) = area_events.iter().next() else { return };
    let Some(local) = local_entity.0 else { return };
    if event.target != local || event.overlap == AreaOverlapType::Entered { return }
    let Ok(_) = sensors.get(event.sensor) else { return };
    let Ok(inv) = buyers.get(local) else { return };
        
    snapshot.inventory = inv.clone();
    dbg!(inv.clone());
    snapshot.gold = 0;
}

#[derive(Resource, Default)]
pub struct StoreSnapshot{
    inventory: Inventory,
    gold: i32,
}

#[derive(Event)]
pub struct UndoPressEvent{
    pub entity: Entity
}

#[derive(Event)]
pub struct StoreEvent{
    pub item: Item,
    pub player: Entity,
    pub direction: TransactionType,
}

#[derive(PartialEq, Eq)]
pub enum TransactionType{
    Buy, 
    Sell,
}