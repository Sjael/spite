use crate::prelude::*;

pub const PLAYER: u32 = 1 << 0;
pub const ABILITY: u32 = 1 << 1;
pub const GROUND: u32 = 1 << 2;
pub const WALL: u32 = 1 << 3;
pub const FLUFF: u32 = 1 << 4;

pub trait CollisionLayersGameDefs {
    const PLAYER: CollisionLayers;
    const ABILITY: CollisionLayers;
    const WALL: CollisionLayers;
    const GROUND: CollisionLayers;
    const FLUFF: CollisionLayers;
}

impl CollisionLayersGameDefs for CollisionLayers {
    const PLAYER: CollisionLayers = CollisionLayers::from_bits(PLAYER, PLAYER | WALL);
    const GROUND: CollisionLayers = CollisionLayers::from_bits(GROUND, u32::MAX);
    const WALL: CollisionLayers = CollisionLayers::from_bits(WALL, u32::MAX);
    const ABILITY: CollisionLayers = CollisionLayers::from_bits(ABILITY, PLAYER | WALL);
    const FLUFF: CollisionLayers = CollisionLayers::from_bits(FLUFF, u32::MAX);
}

pub trait LockedAxesGameDefs {
    const ACTOR: LockedAxes;
}

impl LockedAxesGameDefs for LockedAxes {
    const ACTOR: LockedAxes = LockedAxes::ROTATION_LOCKED.lock_translation_y();
}
