use bevy::{prelude::*, utils::HashMap};

use crate::ability::Ability;

#[derive(Reflect, Debug, Clone)]
pub struct Rank {
    pub current: u8,
    pub max: u8,
}

impl Default for Rank {
    fn default() -> Self {
        Rank { current: 0, max: 5 }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct AbilityRanks {
    pub map: HashMap<Ability, Rank>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct AbilityMap {
    pub ranks: HashMap<Ability, u32>,
    pub cds: HashMap<Ability, Timer>,
}
