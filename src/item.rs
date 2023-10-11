use lazy_static::lazy_static;
use std::collections::HashMap;

use derive_more::Display;

use bevy::prelude::*;

use crate::{actor::stats::Stat, assets::Items, ui::inventory::Inventory};

#[derive(Component, Reflect, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum Item {
    Arondight,
    #[default]
    SoulReaver,
    HiddenDagger,
    BookOfSouls,
    Witchblade,
    Polynomicon,
    DruidStone,
    Deathbringer,
}

#[derive(Default, Clone)]
pub struct ItemInfo {
    /// Cost of this item, excluding parts.
    pub price: u32,
    /// Direct parts to this item.
    pub parts: Vec<Item>,
    pub stats: HashMap<Stat, u32>,
    // pub passives: Vec<ItemPassive>,
}

// stuff that isn't per 'stage' of an item, downstream of hierarchy
#[derive(Default, Clone)]
pub struct ItemTotal {
    /// Total cost of this item, including parts.
    pub total_price: u32,
    /// Flattened parts related to this item.
    pub flat_parts: Vec<(u8, Item)>,
    pub ancestors: Vec<Item>,
}

lazy_static! {
    pub static ref ITEM_DB: HashMap<Item, ItemInfo> = {
        use Stat::*;
        use Item::*;
        HashMap::from([
            (
                Arondight,
                ItemInfo{
                    price: 100,
                    parts: vec![SoulReaver],
                    stats: HashMap::from([
                        (PhysicalPower, 60),
                        (CooldownReduction, 15),
                    ]),
                }
            ),
            (
                SoulReaver,
                ItemInfo{
                    price: 100,
                    parts: vec![Polynomicon, Polynomicon],
                    stats: HashMap::from([
                        (PhysicalPower, 60),
                        (CooldownReduction, 15),
                    ]),
                }
            ),
            (
                Deathbringer,
                ItemInfo{
                    price: 900,
                    parts: vec![HiddenDagger, HiddenDagger],
                    stats: HashMap::from([
                        (PhysicalPower, 60),
                        (PhysicalPenetration, 15),
                    ]),
                }
            ),
            (
                HiddenDagger,
                ItemInfo{
                    price: 500,
                    stats: HashMap::from([
                        (PhysicalPower, 15),
                    ]),
                    ..default()
                }
            ),
            (
                BookOfSouls,
                ItemInfo{
                    price: 100,
                    stats: HashMap::from([
                        (MagicalPower, 30),
                    ]),
                    ..default()
                }
            ),
            (
                DruidStone,
                ItemInfo{
                    price: 300,
                    stats: HashMap::from([
                        (PhysicalProtection, 20),
                    ]),
                    ..default()
                }
            ),
            (
                Polynomicon,
                ItemInfo{
                    price: 100,
                    stats: HashMap::from([
                        (MagicalPower, 80),
                        (CooldownReduction, 20),
                    ]),
                    parts: vec![BookOfSouls, BookOfSouls],
                }
            ),
        ])
    };
    // Creates both the total cost of the item, and the total list of components for easy subtraction of discounts
    pub static ref ITEM_TOTALS: HashMap<Item, ItemTotal> = {
        let mut map = HashMap::new();
        for (item, info) in ITEM_DB.clone().into_iter(){
            if !map.contains_key(&item){
                map.insert(item.clone(), item.calculate_total());
            }

            for part in info.parts{
                let part_info: &mut ItemTotal = map.entry(part.clone()).or_insert(part.calculate_total());
                if part_info.ancestors.contains(&item){ continue };
                part_info.ancestors.push(item.clone());
            }
        }
        map
    };
}

impl Item {
    pub fn get_image(&self, images: &Res<Items>) -> UiImage {
        use Item::*;
        let image = match self {
            Arondight => images.arondight.clone(),
            SoulReaver => images.soul_reaver.clone(),
            HiddenDagger => images.hidden_dagger.clone(),
            Witchblade => images.witchblade.clone(),
            BookOfSouls => images.book_of_souls.clone(),
            Polynomicon => images.polynomicon.clone(),
            DruidStone => images.druid_stone.clone(),
            Deathbringer => images.witchblade.clone(),
            _ => images.hidden_dagger.clone(),
        };
        image.into()
    }

    /// List of common parts between this item's set of flat parts and a list of
    /// items.
    pub fn common_parts(&self, items: impl Iterator<Item = Item>) -> Vec<Item> {
        let mut all_parts = self.flat_parts();
        all_parts.sort_by(|(a, _), (b, _)| a.cmp(b));

        let bong = items.collect::<Vec<_>>();
        let bing = bong.iter().rev().cloned().collect::<Vec<_>>();
        let mut items = bing;

        let mut common = Vec::new();

        while all_parts.len() > 0 {
            let (_, component) = all_parts.remove(0);

            if let Some(item_index) = items.iter().position(|x| x == &component) {
                items.remove(item_index);
            } else {
                // Don't remove subparts
                continue
            };

            common.push(component);

            // If we have the item then remove all subparts
            for (_, part) in component.flat_parts() {
                if let Some(part_index) = all_parts.iter().position(|(_, x)| x == &part) {
                    all_parts.remove(part_index);
                }
            }
        }

        common
    }

    pub fn discounted_price(&self, inventory: &Inventory) -> u32 {
        let discount: u32 = self.common_parts(inventory.items()).iter().map(|component| component.total_price()).sum();
        self.total_price() - discount
    }

    fn calculate_total(&self) -> ItemTotal {
        let info = self.info();
        let mut total_price = info.price;
        let mut flat_parts = info.parts.iter().map(|i| (0u8, *i)).collect::<Vec<_>>();
        for part in info.parts {
            let mut part_total = part.calculate_total();
            total_price += part_total.total_price;
            flat_parts.append(&mut part_total.flat_parts);
        }
        ItemTotal {
            total_price,
            flat_parts,
            ancestors: Vec::new(),
        }
    }

    pub fn total(&self) -> ItemTotal {
        ITEM_TOTALS.get(self).cloned().expect(&format!("Item total doesn't exist for {:?}", self))
    }
    pub fn info(&self) -> ItemInfo {
        ITEM_DB.get(self).cloned().expect(&format!("Item info doesn't exist for {:?}", self))
    }

    pub fn parts(&self) -> Vec<Item> {
        self.info().parts
    }
    pub fn flat_parts(&self) -> Vec<(u8, Item)> {
        self.total().flat_parts
    }
    pub fn total_price(&self) -> u32 {
        self.total().total_price
    }
    pub fn price(&self) -> u32 {
        self.info().price
    }
    pub fn ancestors(&self) -> Vec<Item> {
        self.total().ancestors
    }
}
