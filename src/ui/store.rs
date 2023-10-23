use bevy::{ecs::system::Command, prelude::*};
use bevy_rapier3d::rapier::prelude::ChannelEventCollector;

use crate::{
    ability::TargetsInArea,
    actor::{
        player::PlayerEntity,
        stats::{Attributes, Stat, Stat::*},
    },
    area::{AreaOverlapEvent, AreaOverlapType},
    assets::Items,
    game_manager::{Fountain, GameModeDetails},
    inventory::Inventory,
    item::Item,
};

use super::{
    styles::{NORMAL_BUTTON, PRESSED_BUTTON},
    ui_bundles::*,
    SpectatingSet,
};

pub struct StorePlugin;
impl Plugin for StorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CategorySorted::default());
        app.insert_resource(ItemInspected::default());
        app.insert_resource(StoreSnapshot::default());
        app.add_event::<StoreEvent>();
        app.add_event::<UndoPressEvent>();

        app.add_systems(
            Update,
            (
                sort_items,
                click_category,
                inspect_item,
                try_buy_item,
                try_sell_item,
                try_undo_store,
                leave_store,
                update_discounts.after(inspect_item),
            )
                .in_set(SpectatingSet),
        );
    }
}

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
    mut item_query: Query<(&ItemAttributes, &mut Style)>,
    categories_toggled: Res<CategorySorted>,
) {
    if !categories_toggled.is_changed() {
        return;
    }
    for (attributes, mut style) in item_query.iter_mut() {
        style.display = Display::default();
        if categories_toggled.0.is_empty() {
            continue;
        }
        if attributes
            .0
            .iter()
            .any(|stat| categories_toggled.0.contains(stat))
        {
            continue;
        }
        style.display = Display::None;
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
        if *interaction != Interaction::Pressed {
            continue;
        };
        for mut color in &mut button_query {
            *color = NORMAL_BUTTON.into();
        }
        let Ok(mut color) = button_query.get_mut(entity) else { continue };
        if !categories_toggled.0.contains(&category.0) {
            categories_toggled.0 = vec![category.0.clone()];
            *color = PRESSED_BUTTON.into();
        } else {
            categories_toggled.0 = Vec::new();
        }
        return;
    }
}

fn update_discounts(
    mut query: ParamSet<(
        Query<(&mut Text, &ItemDiscount), Changed<ItemDiscount>>,
        Query<(&mut Text, &ItemDiscount)>,
    )>,
    player_entity: Res<PlayerEntity>,
    changed_inventories: Query<(&Inventory, Entity), Changed<Inventory>>,
    inventories: Query<&Inventory>,
) {
    let Some(local) = player_entity.0 else { return };
    for (inv, entity) in changed_inventories.iter() {
        if entity != local {
            continue;
        }
        for (mut text, discounted_item) in query.p1().iter_mut() {
            discount_style(discounted_item.0.clone(), &inv, &mut text);
        }
    }
    for (mut text, discounted_item) in query.p0().iter_mut() {
        let Ok(inv) = inventories.get(local) else { continue };
        discount_style(discounted_item.0.clone(), &inv, &mut text);
    }
}

pub fn update_inventory(
    mut changed_inventories: Query<(&Inventory, &mut Attributes), Changed<Inventory>>,
) {
    for (inv, mut attributes) in changed_inventories.iter_mut() {}
}

fn discount_style(item: Item, inv: &Inventory, text: &mut Text) {
    let price = item.total_price();
    let discount = item.discounted_price(&inv.clone());
    text.sections[0].value = discount.to_string();
    if price > discount {
        text.sections[0].style.color = Color::GREEN
    } else {
        text.sections[0].style.color = Color::WHITE
    }
}

pub fn inspect_item(
    mut commands: Commands,
    mut interaction_query: Query<(&Item, &Interaction), (Changed<Interaction>, With<StoreItem>)>,
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
        if *interaction != Interaction::Pressed {
            continue;
        }
        if item_inspected.0 == Some(item.clone()) {
            continue;
        } // already inspecting item

        let Ok((tree_entity, tree_children)) = tree_holder.get_single() else { return };
        item_inspected.0 = Some(item.clone());
        if let Some(children) = tree_children {
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
        }
        let tree = spawn_tree(&mut commands, item.clone(), &items);
        commands.entity(tree_entity).push_children(&[tree]);

        let Ok((parents_entity, parents_children)) = parents_holder.get_single() else { return };
        if let Some(children) = parents_children {
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
        }
        commands.entity(parents_entity).with_children(|parent| {
            let ancestors = item.ancestors();
            for ancestor in ancestors {
                parent.spawn(store_item(&items, ancestor));
            }
        });

        if let Ok(mut price_text) = text_set.p0().get_single_mut() {
            price_text.sections[0].value = item.total_price().to_string();
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
    let parts = item.info().parts;
    if !parts.is_empty() {
        let hori = commands.spawn(hori()).id();
        commands.entity(vert).push_children(&[hori]);
        for part in parts {
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
) {
    for event in events.iter() {
        if event.direction != TransactionType::Buy {
            continue;
        }
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.player) else { continue };
        let wallet = attributes.get(Stat::Gold);
        let discounted_price = event.item.discounted_price(&inventory);
        if wallet > discounted_price {
            // remove components
            for item in event.item.common_parts(inventory.items()) {
                if inventory.take(item) {
                    attributes.remove_stats(item.info().stats.into_iter());
                }
            }

            if inventory.insert(event.item) {
                // pay the price
                let gold = attributes.get_mut(Stat::Gold);
                *gold -= discounted_price;
                snapshot.gold -= discounted_price;
            } else {
                info!("inventory full");
            }
        } else {
            info!("you're too broke for that lol");
        }
    }
}

fn try_sell_item(
    mut events: EventReader<StoreEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    mut snapshot: ResMut<StoreSnapshot>,
) {
    for event in events.iter() {
        if event.direction != TransactionType::Sell {
            continue;
        }
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.player) else { continue };

        let gold = attributes.get_mut(Stat::Gold);
        let refund = event.item.total_price();
        if let Some(index) = inventory
            .iter()
            .position(|old| *old == Some(event.item.clone()))
        {
            inventory.0[index] = None;
            let sell_price = (refund as f32 * 0.67).floor();
            *gold += sell_price;
            attributes.remove_stats(event.item.info().stats.into_iter());
            snapshot.gold += sell_price;
        }
    }
}

fn try_undo_store(
    mut events: EventReader<UndoPressEvent>,
    mut buyers: Query<(&mut Attributes, &mut Inventory)>,
    mut snapshot: ResMut<StoreSnapshot>,
) {
    for event in events.iter() {
        let Ok((mut attributes, mut inventory)) = buyers.get_mut(event.entity) else { continue };
        let gold = attributes.get_mut(Stat::Gold);
        *inventory = snapshot.inventory;
        *gold -= snapshot.gold;
        snapshot.gold = 0.0;
    }
}

fn leave_store(
    mut snapshot: ResMut<StoreSnapshot>,
    buyers: Query<&Inventory>,
    local_entity: Res<PlayerEntity>,
    mut area_events: EventReader<AreaOverlapEvent>,
    sensors: Query<&TargetsInArea, With<Fountain>>,
) {
    let Some(event) = area_events.iter().next() else { return };
    let Some(local) = local_entity.0 else { return };
    if event.target != local || event.overlap == AreaOverlapType::Entered {
        return;
    }
    let Ok(_) = sensors.get(event.sensor) else { return };
    let Ok(inv) = buyers.get(local) else { return };

    snapshot.inventory = inv.clone();
    dbg!(inv.clone());
    snapshot.gold = 0.0;
}

impl Command for StoreEvent {
    fn apply(self, world: &mut World) {
        let mut counter = world.get_resource_or_insert_with(GameModeDetails::default);
        let mut query = world.query::<(&mut Attributes, &Inventory)>();
        for (x, y) in query.iter_mut(world) {}
        if self.direction == TransactionType::Buy {
        } else {
        }
    }
}

#[derive(Resource, Default)]
pub struct StoreSnapshot {
    inventory: Inventory,
    gold: f32,
}

#[derive(Component, Default)]
pub struct StoreHistory(pub Vec<StoreEvent>);

#[derive(Event)]
pub struct UndoPressEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct StoreEvent {
    pub item: Item,
    pub player: Entity,
    pub direction: TransactionType,
}

#[derive(PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
}
