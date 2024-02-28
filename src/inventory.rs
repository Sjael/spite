use bevy::prelude::*;

use crate::{
    actor::player::LocalPlayer,
    assets::Items,
    item::Item,
    prelude::InGameSet,
    ui::{holding::DropType, ui_bundles::{item_image_build, BuildSlotNumber}},
};

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inventory>();

        app.add_systems(
            Update,
            (update_inventory_ui, ).in_set(InGameSet::Update),
        );
    }
}

// TODO
// HIDE ONLY ITEMS IN LIST node

fn update_inventory_ui(
    mut commands: Commands,
    query: Query<(&Inventory, Entity), Changed<Inventory>>,
    slot_query: Query<(Entity, &BuildSlotNumber)>,
    items: Res<Items>,
    local_entity: Option<Res<LocalPlayer>>,
) {
    let Some(local) = local_entity else { return };
    for (inv, entity) in query.iter() {
        dbg!(inv.clone());
        if entity != *local {
            continue
        }
        for (slot_e, index) in &slot_query {
            commands.entity(slot_e).despawn_descendants();
            let Some(item) = inv.get(index.0 as usize - 1).unwrap_or(&None) else { continue };
            let new_item = commands.spawn(item_image_build(&items, item.clone())).id();
            commands.entity(new_item).set_parent(slot_e);
        }
    }
}

fn update_inventory_parents(
    mut inventories: Query<&mut Inventory>,
    local_entity: Option<Res<LocalPlayer>>,
    ui_query: Query<&Parent, (With<Item>, With<DropType>, Changed<Parent>)>,
    parent_query: Query<&BuildSlotNumber>,
) {
    let Some(local) = local_entity else { return };
    let Ok(mut inv) = inventories.get_mut(**local) else { return };
    let mut swapping = Vec::new();
    for parent in &ui_query {
        let Ok(index) = parent_query.get(parent.get()) else { continue };
        swapping.push(index.0);
        if swapping.len() == 2 {
            inv.swap(swapping[0] as usize, swapping[1] as usize);
            break
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct Inventory {
    items: [Option<Item>; 6],
}

impl Inventory {
    /// Iterator over all items in this inventory.
    pub fn items(&self) -> impl Iterator<Item = Item> + '_ {
        self.iter().cloned().filter_map(|x| x)
    }

    /// Take the first instance of this item from the inventory.
    pub fn take(&mut self, item: Item) -> bool {
        if let Some(index) = self.iter().position(|old| *old == Some(item)) {
            self[index] = None;
            true
        } else {
            false
        }
    }

    /// Insert this item into the first available slot.
    pub fn insert(&mut self, item: Item) -> bool {
        if let Some(index) = self.iter().position(|old| *old == None) {
            self[index] = Some(item);
            true
        } else {
            false
        }
    }
}
