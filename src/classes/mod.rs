use bevy::prelude::*;

use crate::ability::Ability;

pub mod hunter;
pub mod warrior;

pub enum Class {
    Hunter,
    Nomad,
    Berserker,
    Cultist,
    Arbiter,
    Shaman,
    Sage,
    Mender,
    Entropist,
    Mercenary,
}

impl Class {
    pub fn ability_roster(&self) -> Vec<Ability> {
        match self {
            Class::Hunter => vec![Ability::Bomb, Ability::Dash],
            _ => Vec::new(),
        }
    }
}

// Resource is what thematically 'fuels' each of the classes
// another way to put it: what they believe in, what will deliver them salvation
// Each system works different fundamentally, mostly in how 'points' are primarily
// Classes will also have abilities that generate their respective resource, 'priming' abilities
#[derive(Component, PartialEq, Eq)]
pub enum ResourceKind {
    // The hunt, survival, simple provisions and necessity
    // 6 points, only generates on a kill, full for player kills
    Instinct,
    // Movement and temporality, traversing landscape, discovery, instinct without concrete direction
    // 5 points, passively generate with movement
    Energy,
    // Chaos and destruction, not strictly the amount but each iteration
    // 5 points, passively generate when taking or dealing damage
    Fury,
    // Other beings' lifeforce, not generated on their own but simply to drain others, quantity matters not iterations
    // 6 points, generates from a % of damage dealt (every 20% of x's max hp dealt, gain 1 blood vial)
    Blood,
    // The rules/restrictions placed on oneself
    // No points, pick a condition that must be met, once it is fulfilled you get a big bonus (full hp bar, full cooldown reset)
    Pillars,
    // The current of 'aurora' flowing through them, like a 'heat up' mechanic
    // 4 points, generates from most abilities, abilities gain effects when in higher flow, huge decay
    Flow,
    // Homeostatis, fairness, 2 parts of a whole, respecting a cycle, maintaining balance
    // 5 points, could have huge buffs for being near 50% (3), but also for hitting both extremes quickly? (cycling)
    Balance, // Unnamed resource?
    // Altruism, sacrifice, foil to Cultist, wants to give away lifeforce
    // 5 points, generates passively w/ higher hp
    Compassion,
    // Random chance, math is ultimate fairness, perfectly 'moral' in some sense, as a leader it cannot 'betray' you
    // No points, flip coin / roll the bones core ability that affects huge parts of the kit
    Luck,
    // Money, 'value', productivity
    // Gold, you legit spend your own gold to use abilities, might be broken
    Gold,
}

// It's important to note the difference between what each class fights 'for' and what fuels them
// Some classes have a very cyclic nature to them, berserker, cultist, monk, in that what they fight for feeds back to what fuels them
// This resource is strictly what 'fuels' them, not necessarily the goal they are trying to reach
