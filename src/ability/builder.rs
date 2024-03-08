use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    ability::{shape::AbilityShape, Ability, TagInfo},
    buff::BuffInfo,
    crowd_control::{CCInfo, CCKind},
    stats::Stat,
};

impl Ability {
    fn get_base_info(&self) -> AbilityInfo {
        use Ability::*;
        match self {
            Frostbolt => AbilityInfo {
                cooldown: 4.5,
                cost: 0,
                stages: HashMap::from([(
                    Trigger::Cast,
                    AbilityStage::DeployArea(DeployStage {
                        origin: CastOrigin::Caster,
                        shape: AbilityShape::Rectangle {
                            length: 0.8,
                            width: 0.5,
                        },
                        tags: vec![
                            TagInfo::Damage(38.0),
                            TagInfo::CC(CCInfo {
                                cckind: CCKind::Stun,
                                duration: 1.0,
                            }),
                            TagInfo::Buff(BuffInfo {
                                stat: Stat::Health.into(),
                                amount: 10.0,
                                duration: 10.0,
                                ..default()
                            }),
                        ],
                        movement: Some(Path {
                            kind: PathKind::Straight,
                            speed: 1.0,
                            lifetime: 2.0,
                        }),
                    }),
                )]),
            },
            _ => AbilityInfo::default(),
        }
    }

    fn get_changes_per_rank(&self) -> Vec<PerRankChange> {
        use Ability::*;
        use PerRankChange::*;
        match self {
            Frostbolt => vec![Cooldown(1.0), BaseDamage(40)],
            _ => Vec::new(),
        }
    }

    fn get_overrides(&self) -> Vec<Override> {
        Vec::new()
    }

    fn get_info_at_rank(&self, rank: u32) -> AbilityInfo {
        let mut info = self.get_base_info();

        let changes = self.get_changes_per_rank();
        for change in changes {
            let diff = change.get_at_rank(rank);
            // subtract or add diff to relevant info field
            // need a method on abilityinfo that matches PerRankChange
        }
        info
    }
}

// Need to put all ranks into here for good tooltips
pub struct AbilityBlueprint {
    pub blueprint: HashMap<u8, AbilityInfo>,
}

impl AbilityBlueprint {
    pub fn new(ability: Ability) -> Self {
        let mut blueprint = HashMap::new();
        for i in 1..=5 {
            blueprint.insert(i as u8, ability.get_info_at_rank(i));
        }
        Self { blueprint }
    }
}

#[derive(Default)]
pub struct AbilityInfo {
    pub stages: HashMap<Trigger, AbilityStage>,
    pub cooldown: f32,
    pub cost: u32,
}

// The fundemental types of abilities, sorted by how often they will likely to design around
// (you can describe at least 80% of abilities in smite in the first 2, either make a collider or mobility)
pub enum AbilityStage {
    DeployArea(DeployStage), // spawns a collider that does shit
    // Mobility(MobilityStage), // Moves self (aka applies a cc to self?)
    // Target(TargetStage), // Applies a buff / CC / deploy to a target, needs a target to even fire (could be combined with 'Buff')
    // Stance // (changes kit on self, maybe instead acts as a buff? nah prob a state change with a timer kinda)
    // Detonate // (affects another element, like a deployed ability or stacks of a debuff)
    Buff(BuffInfo), // Only applies a buff to self
}

pub struct DeployStage {
    pub origin: CastOrigin,
    pub shape: AbilityShape,
    pub tags: Vec<TagInfo>,
    pub movement: Option<Path>,
}

// LATER
// pub struct FxInfo {
//     anim: AnimationClip,
//     audio: AudioSink,
// }

pub struct MobilityStage {
    pub kind: MobilityKind,
    pub omni: bool,
    pub distance: f32,
    pub wall_pen: bool,
}

pub enum MobilityKind {
    Dash { speed: f32 }, // Add Path later?
    Jump { speed: f32, height: f32 },
    Teleport,
}

// Things that can change per rank
pub enum PerRankChange {
    Cooldown(f32),
    BaseDamage(u32),
    Scaling { stat: Stat, amount: u32 },
    Cost(u32),
    // CC(CCInfo),
    // Shape(AbilityShape),
    CastTime(f32),
    Range(f32),
}

impl PerRankChange {
    fn get_at_rank(self, rank: u32) -> f32 {
        use PerRankChange::*;
        let per_rank = match self {
            Scaling { amount, .. } => amount as f32,
            Cooldown(x) | CastTime(x) | Range(x) => x,
            Cost(x) | BaseDamage(x) => x as f32,
        };
        per_rank * (rank - 1) as f32
    }
}

// At this specific rank, set to this
// In smite some abilities say 'at rank 3 this knocks up' for instance
pub struct Override {
    rank: u8,
    change: PerRankChange,
}

// Which way abilities can fire, in order of occurence
#[derive(PartialEq, PartialOrd, Eq, Hash)]
pub enum Trigger {
    Cast,
    Collision,      // Merlin Frostbolt explosion // change to be the bitmask of walls/allies/enemies
    Recast,         // Isis Spirit ball, Thor 1
    TimeDelay(u32), //
    Passive,        // No trigger, buffing all the time
}

// Where to spawn a Deploy ability stage when cast (does it emit from the character or is it lobbed?)
#[derive(PartialEq, PartialOrd, Eq, Hash)]
pub enum CastOrigin {
    Caster,
    Reticle,
    Following(Entity), // For when something adds an area on collision?
}

pub struct Path {
    kind: PathKind,
    speed: f32,
    lifetime: f32,
}

// How abilities can move when spawned
pub enum PathKind {
    Straight,
    Arc,
    // FollowingReticle
}
