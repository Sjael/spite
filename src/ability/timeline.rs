use std::{collections::HashMap, time::Duration};
use bevy::prelude::*;

use super::Ability;

// Maybe we should have a "general" lifetime too even if the thing isn't being
// casted...?
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct AreaLifetime {
    pub seconds: f64,
}
// Ticks, TickInterval, TickBehavior, and DeployAreaStage::Firing all interlink data

#[derive(Component, Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Component)]
pub struct AreaTimeline {
    pub stage: DeployAreaStage,
    pub timer: Timer,
    pub blueprint: HashMap<DeployAreaStage, f32>,
}

impl AreaTimeline {
    pub fn new(bp: HashMap<DeployAreaStage, f32>) -> Self {
        let x = bp.get(&DeployAreaStage::Windup).unwrap_or(&0.0).clone();
        Self {
            blueprint: bp,
            stage: DeployAreaStage::Windup,
            timer: Timer::new(Duration::from_secs_f32(x), TimerMode::Once),
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash, Default)]
pub enum DeployAreaStage {
    Windup,  // between fire from player, to spawned in world
    Outline, // between fire and shown to enemies, change to different component?
    #[default]
    Firing, // active in world
    Recovery, // delay before despawn
}

impl DeployAreaStage {
    pub fn get_next_stage(&mut self) -> Self {
        use DeployAreaStage::*;
        match self {
            Windup => Outline,
            Outline => Firing,
            Firing | Recovery => Recovery,
        }
    }
}

#[derive(Component)]
pub struct OutlineDelay(pub f32);


#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
struct CastingStages(HashMap<Ability, CastTimeline>);

#[derive(Reflect, Clone, Debug)]
struct CastTimeline {
    stage: ActorCastStage,
    timer: Timer,
    blueprint: HashMap<ActorCastStage, f32>,
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum ActorCastStage {
    Charging,   // if ability has a charge before even getting a targetter (athena dash, tia 1)
    Prefire,    // between press and firing/goes on CD
    Channeling, // if channelling an ability, like anubis 1
    Postfire,   // recovery time after going on CD
}