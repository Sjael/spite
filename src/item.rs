

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
}

#[derive(Default, Clone)]
pub struct ItemInfo{
    pub cost: u32,
    pub parts: Vec<Item>,
    pub stats: HashMap<Stat, u32>,
}

lazy_static! {
    pub static ref ITEM_DB: HashMap<Item, ItemInfo> = {
        use Stat::*;
        use Item::*;
        HashMap::from([
            (
                Arondight, 
                ItemInfo{
                    cost: 900,
                    parts: vec![HiddenDagger, HiddenDagger],
                    stats: HashMap::from([
                        (PhysicalPower, 60),
                        (CooldownReduction, 15),
                    ]),
                }
            ),
            (
                SoulReaver, 
                ItemInfo{
                    cost: 700,
                    parts: vec![HiddenDagger, BookOfSouls],
                    stats: HashMap::from([
                        (MagicalPower, 60),
                        (MagicalPenetration, 15),
                    ]),
                }
            ),
            (
                HiddenDagger, 
                ItemInfo{
                    cost: 500,
                    stats: HashMap::from([
                        (PhysicalPower, 15),
                    ]),
                    ..default()
                }
            ),
            (
                BookOfSouls, 
                ItemInfo{
                    cost: 450,
                    stats: HashMap::from([
                        (MagicalPower, 30),
                    ]),
                    ..default()
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
            _ => images.hidden_dagger.clone(),
        };
        image.into()
    }

    pub fn calculate_cost(&self) -> u32 {
        let info = ITEM_DB.get(self).cloned().unwrap_or_default();
        let mut price = info.cost;
        for part in info.parts {
            price += part.calculate_cost();
        }
        price
    }

    pub fn get_parts(&self) -> Vec<Item> {
        ITEM_DB.get(self).cloned().unwrap_or_default().parts
    }

    pub fn get_ancestors(&self) -> Vec<Item> {
        ITEM_ANCESTORS.get(self).cloned().unwrap_or_default()
    }
    
}