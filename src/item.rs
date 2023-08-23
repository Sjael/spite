

use std::collections::HashMap;
use lazy_static::lazy_static;

use derive_more::Display;

use bevy::prelude::*;

use crate::{actor::stats::Stat, assets::Items};

#[derive(Component, Reflect, Clone, Debug, Default, Display, Eq, PartialEq, Hash)]
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
}

#[derive(Default, Clone)]
pub struct ItemInfo{
    pub price: u32,
    pub parts: Vec<Item>,
    pub stats: HashMap<Stat, u32>,
}

#[derive(Default, Clone)]
pub struct ItemTotal{
    pub price: u32,
    pub parts: Vec<Item>,
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
                    parts: vec![HiddenDagger, DruidStone],
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
                    parts: vec![HiddenDagger, Polynomicon],
                    stats: HashMap::from([
                        (MagicalPower, 60),
                        (MagicalPenetration, 15),
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
                Polynomicon, 
                ItemInfo{
                    price: 1150,
                    stats: HashMap::from([
                        (MagicalPower, 80),
                        (CooldownReduction, 20),
                    ]),
                    parts: vec![BookOfSouls, BookOfSouls],
                }
            ),
        ])
    };
    pub static ref ITEM_ANCESTORS: HashMap<Item, Vec<Item>> = {
        let mut map = HashMap::new();
        for (item, info) in ITEM_DB.clone().into_iter(){
            for part in info.parts{
                let ancestors: &mut Vec<Item> = map.entry(part.clone()).or_default();
                if ancestors.contains(&item){ continue };
                ancestors.push(item.clone());
            }
        }
        map
    };
    pub static ref ITEM_TOTALS: HashMap<Item, ItemTotal> = {
        let mut map = HashMap::new();
        for (item, _) in ITEM_DB.clone().into_iter(){
            map.insert(item.clone(), item.calculate_totals());
        }
        map
    };
}

impl Item{

    pub fn get_categories(&self) -> Vec<Stat>{
        ITEM_DB.get(self)
            .cloned()
            .unwrap_or_default()
            .stats.keys()
            .map(|i| *i)
            .collect::<Vec<_>>()
    }

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
            _ => images.hidden_dagger.clone(),
        };
        image.into()
    }

    pub fn calculate_price(&self) -> u32 {
        let info = ITEM_DB.get(self).cloned().unwrap_or_default();
        let mut price = info.price;
        for part in info.parts {
            price += part.calculate_price();
        }
        price
    }
    pub fn calculate_totals(&self) -> ItemTotal {
        let info = ITEM_DB.get(self).cloned().unwrap_or_default();
        let mut price = info.price;
        let mut total_parts = info.parts.clone();
        for part in info.parts {
            let mut sub_total = part.calculate_totals();
            price += sub_total.price;
            total_parts.append(&mut sub_total.parts);
        }
        ItemTotal{
            price: price,
            parts: total_parts,
        }
    }

    pub fn get_parts(&self) -> Vec<Item> {
        ITEM_DB.get(self).cloned().unwrap_or_default().parts
    }

    pub fn get_ancestors(&self) -> Vec<Item> {
        ITEM_ANCESTORS.get(self).cloned().unwrap_or_default()
    }
    
}