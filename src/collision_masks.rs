use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    Ability,
    Ground,
    Wall,
    Fluff,
}

// Collision Components
pub const PLAYER_LAYER: CollisionLayers = CollisionLayers::from_bits(
    Layer::Player as u32,
    Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32,
);
pub const GROUND_LAYER: CollisionLayers = CollisionLayers::from_bits(
    Layer::Ground as u32,
    Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32,
);
pub const WALL_LAYER: CollisionLayers = CollisionLayers::from_bits(
    Layer::Wall as u32,
    Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32,
);
pub const ABILITY_LAYER: CollisionLayers = CollisionLayers::from_bits(
    Layer::Ability as u32,
    Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32 | Layer::Ability as u32,
);
