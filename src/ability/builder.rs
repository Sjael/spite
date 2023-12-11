use std::collections::HashMap;
use bevy::prelude::*;
use crate::{stats::Stat, ability::bundles::FireballInfo};

use super::{Ability, shape::AbilityShape, crowd_control::CCType};



pub struct AbilityBlueprint {
    pub base_ability: Ability,
    pub name: String,
    pub stages: HashMap<Trigger, AbilityStage>,
    pub cooldown: Cooldown,
}

pub struct AbilityStage {
    pub effects: HashMap<AbilityComp, RankNumbers>,
    pub shape: AbilityShape,
    pub path: Path,
}

#[derive(PartialEq, PartialOrd)]
pub enum Trigger {
    Cast(TransformOrigin),
    Collision, // Merlin Frostbolt explosion // change to be the bitmask of walls/allies/enemies
    Detonate,  // Isis Spirit ball, Thor 1
    TimeDelay(f32),
}

#[derive(PartialEq, PartialOrd)]
pub enum TransformOrigin {
    Player,
    Reticle,
}

pub enum Path {
    Static,
    Straight { lifetime: f32, speed: f32 },
}

pub enum AbilityComp {
    Scaling(Stat),
    BaseDamage,
    Cooldown,
    Cost,
    CC(CCType),
}

pub struct RankNumbers {
    pub base: u32,
    pub per_rank: u32,
}

pub struct Scaling {
    pub base: u32,
    pub per_rank: u32,
    pub stat: Stat,
}
pub struct BaseDamage {
    pub base: u32,
    pub per_rank: u32,
}
pub struct Cooldown {
    pub base: u32,
    pub per_rank: u32,
}

trait AbilityFactory {
    fn build(ability: Ability) -> u32;
}

#[derive(Debug, Default, Clone)]
pub struct AbilityBuilder {
    //base_ability: Ability,
    scaling: Option<u32>,
}

impl AbilityBuilder {
    pub fn new(_ability: Ability) -> AbilityBuilder {
        Self {
            //base_ability: ability,
            scaling: None,
        }
    }
    pub fn with_scaling(&mut self, scaling: u32) -> &mut Self {
        self.scaling = Some(scaling);
        self
    }
    pub fn build(&mut self, commands: &mut Commands) -> Entity {
        dbg!(self);
        let ability = FireballInfo::fire_bundle(&Transform::default());
        commands.spawn(ability).id()
    }
}