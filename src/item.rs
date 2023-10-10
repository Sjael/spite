

use std::collections::HashMap;
use lazy_static::lazy_static;

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
pub struct ItemInfo{
    pub price: u32,
    pub parts: Vec<Item>,
    pub stats: HashMap<Stat, u32>,
    // pub passives: Vec<ItemPassive>,
}

// stuff that isn't per 'stage' of an item, downstream of hierarchy
#[derive(Default, Clone)]
pub struct ItemTotal{
    pub price: u32,
    pub flat_parts: Vec<Item>,
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
                    price: 900,
                    parts: vec![HiddenDagger, DruidStone, Polynomicon],
                    stats: HashMap::from([
                        (PhysicalPower, 60),
                        (CooldownReduction, 15),
                    ]),
                }
            ),
            (
                SoulReaver, 
                ItemInfo{
                    price: 700,
                    parts: vec![BookOfSouls, Polynomicon, HiddenDagger],
                    stats: HashMap::from([
                        (MagicalPower, 60),
                        (MagicalPenetration, 15),
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
                    price: 450,
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
                    price: 750,
                    stats: HashMap::from([
                        (MagicalPower, 80),
                        (CooldownReduction, 20),
                    ]),
                    parts: vec![BookOfSouls],
                }
            ),
        ])
    };
    // Creates both the total cost of the item, and the total list of components for easy subtraction of discounts
    pub static ref ITEM_TOTALS: HashMap<Item, ItemTotal> = {
        let mut map = HashMap::new();
        for (item, info) in ITEM_DB.clone().into_iter(){
            if !map.contains_key(&item){
                map.insert(item.clone(), item.calculate_totals());
            }
            
            for part in info.parts{
                let part_info: &mut ItemTotal = map.entry(part.clone()).or_insert(part.calculate_totals());
                if part_info.ancestors.contains(&item){ continue };
                part_info.ancestors.push(item.clone());
            }
        }
        map
    };
}

impl Item{

    pub fn get_image(&self, images: &Res<Items>) -> UiImage {
        use Item::*;
        let image = match self{
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

    pub fn calculate_discount(&self, inventory: &Inventory) -> u32 {
        let mut all_parts = self.totals().flat_parts;
        let checked = inventory.0.iter().cloned()
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        let mut subtract = 0;
        for component in checked{
            let index = if let Some(index) = all_parts.iter().position(|x| x == &component){
                index
            } else {
                continue
            };
            subtract += component.totals().price;
            all_parts.remove(index);
            for part in component.info().parts{
                if let Some(part_index) = all_parts.iter().position(|x| x == &part){
                    all_parts.remove(part_index);
                }
            }
        }
        self.totals().price - subtract
    }

    fn calculate_totals(&self) -> ItemTotal {
        let info = ITEM_DB.get(self).cloned().unwrap_or_default();
        let mut price = info.price;
        let mut flat_parts = info.parts.clone();
        for part in info.parts {
            let mut sub_total = part.calculate_totals();
            price += sub_total.price;
            flat_parts.append(&mut sub_total.flat_parts);
        }
        ItemTotal{
            price,
            flat_parts,
            ancestors: Vec::new(),
        }
    }

   
    pub fn totals(&self) -> ItemTotal{
        ITEM_TOTALS.get(self).cloned().unwrap_or_default()
    }
    pub fn info(&self) -> ItemInfo{
        ITEM_DB.get(self).cloned().unwrap_or_default()
    }
  

    
}