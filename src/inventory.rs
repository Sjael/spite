use bevy::prelude::*;

use crate::{
    actor::player::PlayerEntity,
    assets::Items,
    item::Item,
    ui::{
        ui_bundles::{item_image_build, BuildSlotNumber},
        SpectatingSet,
    },
};

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct Inventory(pub [Option<Item>; 6]);

impl Inventory {
    pub fn items(&self) -> impl Iterator<Item = Item> + '_ {
        self.iter().cloned().filter_map(|x| x)
    }
}

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inventory>();

        app.add_systems(Update, (update_inventory_ui,).in_set(SpectatingSet));
    }
}

// TODO
// HIDE ONLY ITEMS IN LIST node

fn update_inventory_ui(
    mut commands: Commands,
    query: Query<(&Inventory, Entity), Changed<Inventory>>,
    slot_query: Query<(Entity, &BuildSlotNumber)>,
    items: Res<Items>,
    local_entity: Res<PlayerEntity>,
) {
    let Some(local) = local_entity.0 else { return };
    for (inv, entity) in query.iter() {
        if entity != local {
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
