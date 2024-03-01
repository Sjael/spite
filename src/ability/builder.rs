use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    ability::{shape::AbilityShape, Ability}, buff::BuffInfo, crowd_control::CCKind, stats::Stat
};

impl Ability{
    fn get_info(&self, rank: u32) -> AbilityInfo{
        match self {
            Ability::Frostbolt => {}
            _ => (),
        }

        AbilityInfo{
            stages: HashMap::new(),
            cooldown: self.get_cooldown(),
            cost: self.get_cost() as u32,
            effects: HashMap::new(),
        }
    }
}

pub struct AbilityInfo {
    pub stages: HashMap<Trigger, AbilityStage>,
    pub cooldown: f32,
    pub effects: HashMap<Components, RankNumbers>,
    pub cost: u32,
}

pub enum AbilityStage{
    DeployArea(DeployStage),
    Buff(BuffStage),
    // Target(TargetStage), // Geb Shield
    // Mobility(MobilityStage),
}

pub struct DeployStage {
    pub shape: AbilityShape,
    pub path: Path,
}

pub struct BuffStage{
    pub info: BuffInfo,
    pub fx: FxInfo,
}

pub struct FxInfo{
    anim: AnimationClip,
    audio: AudioSink,
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

pub enum Components {
    Scaling(Stat),
    BaseDamage,
    Cost,
    CC(CCKind),
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
        let ability = Ability::Fireball;
        commands.spawn(ability).id()
    }
}
