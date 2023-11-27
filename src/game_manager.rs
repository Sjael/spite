use std::collections::HashMap;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    ability::{
        bundles::Caster, Ability, MaxTargetsHit, TargetsHittable, TargetsInArea, TickBehavior,
    },
    actor::{
        cast_ability,
        player::{Player, PlayerEntity},
        stats::{Attributes, Stat},
        view::Reticle,
        AbilityRanks, RespawnEvent,
    },
    area::homing::Homing,
    director::GameModeDetails,
    prelude::*,
    GameState,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub struct Team(pub TeamMask);
// Team masks
bitflags::bitflags! {
    #[derive(Reflect, Default)]
    pub struct TeamMask: u32 {
        const ALL = 1 << 0;
        const TEAM_1 = 1 << 1;
        const TEAM_2 = 1 << 2;
        const TEAM_3 = 1 << 3;
        const NEUTRALS = 1 << 4;
    }
}

pub const TEAM_1: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_1.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_2: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_2.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_3: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_3.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_NEUTRAL: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::NEUTRALS.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_ALL: Team = Team(TeamMask::from_bits_truncate(TeamMask::ALL.bits()));

#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    Ability,
    Ground,
    Wall,
    Fluff,
}

// Collision Grouping Flags
pub const PLAYER_LAYER: CollisionLayers =
    CollisionLayers::new([Layer::Player], [Layer::Player, Layer::Wall, Layer::Ground]);
    // CollisionLayers::from_bits(u32::MAX, u32::MAX);

pub const GROUND_LAYER: CollisionLayers =
    CollisionLayers::new([Layer::Ground], [Layer::Player, Layer::Wall, Layer::Ground]);
    // CollisionLayers::from_bits(u32::MAX, u32::MAX);
pub const WALL_LAYER: CollisionLayers =
    CollisionLayers::new([Layer::Wall], [Layer::Player, Layer::Wall, Layer::Ground]);
    // CollisionLayers::from_bits(u32::MAX, u32::MAX);
pub const ABILITY_LAYER: CollisionLayers = CollisionLayers::new(
    [Layer::Ability],
    [Layer::Player, Layer::Wall, Layer::Ground, Layer::Ability],
);

