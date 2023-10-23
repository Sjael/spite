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
        return
    }
    for (attributes, mut style) in item_query.iter_mut() {
        style.display = Display::default();
        if categories_toggled.0.is_empty() {
            continue
        }
        if attributes
            .0
            .iter()
            .any(|stat| categories_toggled.0.contains(stat))
        {
            continue
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
            continue
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
        return
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
            continue
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
            continue
        }
        if item_inspected.0 == Some(item.clone()) {
            continue
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
    mut buyers: Query<(
        &mut Attributes,
        &mut Inventory,
        &mut StoreBuffer,
        &mut StoreHistory,
    )>,
) {
    for event in events.iter() {
        if event.direction != TransactionType::Buy {
            continue
        }
        let Ok((mut attributes, mut inventory, mut buffer, mut history)) =
            buyers.get_mut(event.player)
        else {
            continue;
        };
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
                buffer.insert(event.item);
                if event.fresh {
                    history.insert(*event);
                }
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
    mut buyers: Query<(
        &mut Attributes,
        &mut Inventory,
        &mut StoreBuffer,
        &mut StoreHistory,
    )>,
) {
    for event in events.iter() {
        if event.direction != TransactionType::Sell {
            continue
        }
        let Ok((mut attributes, mut inventory, mut buffer, mut history)) =
            buyers.get_mut(event.player)
        else {
            continue;
        };

        let refund = event.item.total_price();
        if inventory.take(event.item) {
            attributes.remove_stats(event.item.info().stats.into_iter());

            let gold = attributes.get_mut(Stat::Gold);
            let sell_price = if buffer.take_fresh(event.item) {
                refund
            } else {
                (refund as f32 * 0.67).floor()
            };
            *gold += sell_price;
            if event.fresh {
                history.insert(*event);
            }
        }
    }
}

fn try_undo_store(
    mut undo_events: EventReader<UndoPressEvent>,
    mut store_events: EventWriter<StoreEvent>,
    mut buyers: Query<&mut StoreHistory>,
) {
    for event in undo_events.iter() {
        info!("undo event");
        let Ok(mut store_history) = buyers.get_mut(event.entity) else { continue };
        info!("has history: {:?}", store_history);

        if let Some(mut last) = store_history.pop_last() {
            last.fresh = false;
            store_events.send(last.flip());
        }
    }
}

fn leave_store(
    mut buyers: Query<&mut StoreBuffer>,
    local_entity: Res<PlayerEntity>,
    mut area_events: EventReader<AreaOverlapEvent>,
    sensors: Query<&TargetsInArea, With<Fountain>>,
) {
    let Some(event) = area_events.iter().next() else { return };
    let Some(local) = local_entity.0 else { return };
    if event.target != local || event.overlap == AreaOverlapType::Entered {
        return
    }
    let Ok(_) = sensors.get(event.sensor) else { return };
    let Ok(mut buffer) = buyers.get_mut(local) else { return };

    buffer.clear();
}

/// Uncommitted changes to the inventory.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect]
pub struct StoreBuffer(Vec<Item>);

impl StoreBuffer {
    pub fn take_fresh(&mut self, item: Item) -> bool {
        if let Some(index) = self.0.iter().position(|i| *i == item) {
            self.0.swap_remove(index);
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, item: Item) {
        self.0.push(item);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Component, Debug, Default)]
pub struct StoreHistory(Vec<StoreEvent>);

impl StoreHistory {
    pub fn pop_last(&mut self) -> Option<StoreEvent> {
        if self.0.len() == 0 {
            return None;
        }
        Some(self.0.remove(self.0.len() - 1))
    }

    pub fn insert(&mut self, event: StoreEvent) {
        self.0.push(event);
    }
}

#[derive(Event)]
pub struct UndoPressEvent {
    pub entity: Entity,
}

#[derive(Debug, Event, Copy, Clone)]
pub struct StoreEvent {
    pub item: Item,
    pub player: Entity,
    pub direction: TransactionType,
    pub fresh: bool,
}

impl StoreEvent {
    pub fn flip(mut self) -> Self {
        self.direction = self.direction.flip();
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
}

impl TransactionType {
    pub fn flip(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}
