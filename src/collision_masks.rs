use bevy_xpbd_3d::prelude::*;
use bevy::prelude::*;


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub struct Team(pub TeamMask);
// Team masks
bitflags::bitflags! {
    #[derive(Reflect, Default,)]
    pub struct TeamMask: u32 {
        const ALL = 1 << 0;
        const TEAM_1 = 1 << 1;
        const TEAM_2 = 1 << 2;
        const TEAM_3 = 1 << 3;
        const NEUTRALS = 1 << 4;
    }
}

// Team Components 
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

// Collision Components 
pub const PLAYER_LAYER: CollisionLayers =
    CollisionLayers::from_bits(Layer::Player as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);
pub const GROUND_LAYER: CollisionLayers =
    CollisionLayers::from_bits(Layer::Ground as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);
pub const WALL_LAYER: CollisionLayers =
    CollisionLayers::from_bits(Layer::Wall as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32);
pub const ABILITY_LAYER: CollisionLayers = 
    CollisionLayers::from_bits(Layer::Ability as u32, Layer::Player as u32 | Layer::Wall as u32 | Layer::Ground as u32| Layer::Ability as u32);
