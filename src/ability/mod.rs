

use std::{time::Duration, collections::HashMap};

use bevy::prelude::*;
use leafwing_input_manager::Actionlike;
use derive_more::Display;
use crate::{crowd_control::CCInfo, buff::BuffInfo};

use self::bundles::{FrostboltInfo, FireballInfo, DefaultAbilityInfo};

pub mod bundles;
pub mod shape;

#[derive(Actionlike, Component, Reflect, FromReflect, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash,)]
#[reflect(Component)]
pub enum Ability {
    Frostbolt,
    Fireball,
    #[default]
    BasicAttack,
    Dash,
}

impl Ability {
    // cooldown IS BEING USED
    pub fn get_cooldown(&self) -> f32 {
        match self {
            Ability::Dash => 7.,
            Ability::Frostbolt => 3.5,
            Ability::Fireball => 4.,
            Ability::BasicAttack => 2.,
        }
    }

    pub fn get_windup(&self) -> f32{
        match self {
            Ability::Frostbolt => 0.2,
            Ability::Fireball => 0.8,
            _ => 0.4,
        }
    }

    pub fn on_reticle(&self) -> bool{
        match self{
            Ability::Frostbolt => true,
            _ => false,
        }
    }

    pub fn get_bundle(&self, commands: &mut Commands, transform: &Transform) -> Entity{
        match self{
            Ability::Frostbolt => commands.spawn(FrostboltInfo::fire_bundle(transform)).id(),
            Ability::Fireball => commands.spawn(FireballInfo::fire_bundle(transform)).id(),
            _ => commands.spawn(DefaultAbilityInfo::fire_bundle(transform)).id(),
        }
    }

    pub fn get_tooltip(&self) -> AbilityTooltip{
        match self {
            Ability::Frostbolt => AbilityTooltip {
                title: "Frostbolt".to_string(),
                image: "icons/frostbolt.png".to_string(),
                description: "Cold as fuck".to_string(),
            },
            Ability::Dash => AbilityTooltip {
                title: "Driving Strike".to_string(),
                image: "icons/dash.png".to_string(),
                description: "Hercules delivers a mighty strike, driving all enemies back, damaging and Stunning them. Hercules is immune to Knockback during the dash.".to_string(),
            },
            _ => AbilityTooltip {
                title: "Ability".to_string(),
                image: "icons/BasicAttack.png".to_string(),
                description: "A very boring attack".to_string(),
            },
        }}
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AbilityTooltip {
    pub title: String,
    pub image: String,
    pub description: String,
}


// Maybe we should have a "general" lifetime too even if the thing isn't being casted...?
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct CastingLifetime {
    pub seconds: f64,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsInArea {
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct TargetFilter {
    pub target_selection: TargetSelection,
    pub number_of_targets: u8,
}

#[derive(Default, Debug, Clone, FromReflect, Reflect)]
pub enum TargetSelection {
    #[default]
    Closest,
    Random,
    HighestThreat, // Make Threat hashmap of <(Entity, Threat)>
    All,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FilteredTargets {
    pub list: Vec<Entity>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsHittable {
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct MaxTargetsHit {
    pub max: u8,
    pub current: u8,
}

impl MaxTargetsHit {
    pub fn new(max: u8) -> Self {
        Self { max, current: 0 }
    }
}

impl Default for MaxTargetsHit {
    fn default() -> Self {
        Self { max: 255, current: 0 }
    }
}

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct UniqueTargetsHit {
    pub already_hit: Vec<Entity>,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub enum TickBehavior{
    Static(Timer),
    Individual(IndividualTargetTimers),
}

impl Default for TickBehavior{
    fn default() -> Self {
        Self::Static(Timer::default())
    }
}

impl TickBehavior{
    pub fn static_timer() -> Self{
        Self::Static(Timer::new(Duration::from_secs(1), TimerMode::Repeating))    
    }
    pub fn individual() -> Self{
        Self::Individual(IndividualTargetTimers::default())
    }
}

#[derive(Default, Reflect, FromReflect, Debug, Clone)]
pub struct StaticTimer(pub Timer);

// map that holds what things have been hit by an ability instance
#[derive(Default, Debug, Clone, FromReflect, Reflect)]
pub struct IndividualTargetTimers {
    pub map: HashMap<Entity, Timer>,
}

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub struct PausesWhenEmpty;

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub enum Ticks {
    #[default]
    Once,
    Multiple(u32),
    Unlimited,
}

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub struct FiringInterval(pub u32);


#[derive(Component, Default, Clone, Debug)]
pub struct Tags {
    pub list: Vec<TagInfo>,
}

#[derive(Clone, Debug)]
pub enum TagInfo {
    Heal(f32),
    Damage(f32),
    Buff(BuffInfo),
    CC(CCInfo),
    Homing(Ability),
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct HomingInfo {
    projectile_to_spawn: Ability,
}
