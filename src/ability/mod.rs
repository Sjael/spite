//! Construction of abilities.
//!
//! Basic premise of all abilities is there are `collectors`, `filters`, and `appliers`.
//!
//! `Collectors` use colliders/auto-targetters/etc. to determine potential targets for an ability.
//! `Filters` take collectors and disqualify them if they shouldn't be affected.
//! `Appliers` finalize the effects of the ability on the target entity, most commonly damaging or
//! debuffing.

use std::{collections::HashMap, time::Duration};

use crate::{
    assets::Icons,
    prelude::*,
    stats::{Stat, StatsPlugin},
};

use bevy::prelude::*;
use derive_more::Display;
use leafwing_input_manager::Actionlike;

use self::{
    buff::{BuffInfo, BuffPlugin},
    bundles::{BombInfo, DefaultAbilityInfo, FireballInfo, FrostboltInfo},
    cast::CastPlugin,
    crowd_control::{CCInfo, CCPlugin, CCType},
    shape::AbilityShape,
    timeline::{AreaTimeline, DeployAreaStage},
};

pub mod buff;
pub mod builder;
pub mod bundles;
pub mod cast;
pub mod collector;
pub mod crowd_control;
pub mod rank;
pub mod shape;
pub mod timeline;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbilitySet {
    /// Gather entities that should be considered for ability application.
    CollectorUpdate,
    /// Filter/disqualify entities that were collected.
    Filter,
    /// Apply ability effects to entities.
    Apply,
    /// Update filters based on previously collected entities.
    FilterUpdate,
    /// Clear the collected entities for next tick.
    ClearCollected,
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BuffPlugin);
        app.add_plugins(StatsPlugin);
        app.add_plugins(CCPlugin);
        app.add_plugins(CastPlugin);

        app.configure_sets(
            FixedUpdate,
            (
                AbilitySet::CollectorUpdate,
                AbilitySet::Filter,
                AbilitySet::Apply,
                AbilitySet::FilterUpdate,
                AbilitySet::ClearCollected,
            )
                .chain()
                .in_set(InGameSet::Update),
        );

        /*
        app.add_systems(
            FixedUpdate,
            (filter_already_hit, filter_timed_hit).in_set(AbilitySet::Filter),
        );

        app.add_systems(
            FixedUpdate,
            (update_already_hit, update_timed_hit).in_set(AbilitySet::FilterUpdate),
        );

        app.add_systems(
            FixedUpdate,
            clear_collected.in_set(AbilitySet::ClearCollected),
        );
        */
    }
}

#[derive(
    Actionlike, Component, Reflect, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash,
)]
#[reflect(Component)]
pub enum Ability {
    Frostbolt,
    Fireball,
    Bomb,
    #[default]
    BasicAttack,
    Dash,
}

impl Ability {
    pub fn get_cooldown(&self) -> f32 {
        match self {
            Ability::Dash => 7.,
            Ability::Frostbolt => 3.5,
            Ability::Fireball => 4.,
            Ability::BasicAttack => 2.,
            _ => 3.,
        }
    }

    pub fn get_actor_times(&self) -> f32 {
        match self {
            Ability::Frostbolt => 0.2,
            Ability::Fireball => 0.8,
            _ => 0.4,
        }
    }

    pub fn get_area_timeline(&self) -> AreaTimeline {
        let times = match self {
            Ability::Bomb => HashMap::from([
                (DeployAreaStage::Windup, 0.2),
                (DeployAreaStage::Firing, 3.0),
                (DeployAreaStage::Recovery, 0.2),
            ]),
            _ => HashMap::new(),
        };
        let windup = times.get(&DeployAreaStage::Windup).unwrap_or(&0.0).clone();
        AreaTimeline {
            blueprint: times,
            stage: DeployAreaStage::Windup,
            timer: Timer::new(Duration::from_secs_f32(windup), TimerMode::Once),
        }
    }

    pub fn on_reticle(&self) -> bool {
        match self {
            Ability::Fireball => true,
            Ability::Bomb => true,
            _ => false,
        }
    }

    pub fn get_image(&self, icons: &Res<Icons>) -> Handle<Image> {
        match self {
            Ability::Frostbolt => icons.frostbolt.clone(),
            Ability::Fireball => icons.fireball.clone(),
            Ability::Dash => icons.dash.clone(),
            Ability::Bomb => icons.swarm.clone(),
            _ => icons.basic_attack.clone(),
        }
    }

    pub fn get_bundle(&self, commands: &mut Commands, transform: &Transform) -> Entity {
        match self {
            Ability::Frostbolt => commands.spawn(FrostboltInfo::fire_bundle(transform)).id(),
            Ability::Fireball => commands.spawn(FireballInfo::fire_bundle(transform)).id(),
            Ability::Bomb => commands.spawn(BombInfo::fire_bundle(transform)).id(),
            _ => commands
                .spawn(DefaultAbilityInfo::fire_bundle(transform))
                .id(),
        }
    }

    pub fn get_targetter(&self, commands: &mut Commands) -> Entity {
        match self {
            Ability::Frostbolt => commands.spawn(FrostboltInfo::hover_bundle()).id(),
            Ability::Fireball => commands.spawn(FireballInfo::hover_bundle()).id(),
            Ability::Bomb => commands.spawn(BombInfo::hover_bundle()).id(),
            _ => commands.spawn(DefaultAbilityInfo::hover_bundle()).id(),
        }
    }

    pub fn get_tooltip(&self) -> AbilityTooltip {
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
        }
    }

    pub fn damage_type(&self) -> DamageType {
        match self {
            Ability::Frostbolt => DamageType::Magical,
            Ability::Fireball => DamageType::Magical,
            Ability::Bomb => DamageType::Physical,
            _ => DamageType::True,
        }
    }

    pub fn get_scaling(&self) -> u32 {
        match self {
            Ability::Frostbolt => 30,
            _ => 40,
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AbilityTooltip {
    pub title: String,
    pub image: String,
    pub description: String,
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

#[derive(Default, Debug, Clone, Reflect)]
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
        Self {
            max: 255,
            current: 0,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct UniqueTargetsHit {
    pub already_hit: Vec<Entity>,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub enum TickBehavior {
    Static(Timer),
    Individual(IndividualTargetTimers),
}

impl Default for TickBehavior {
    fn default() -> Self {
        Self::Static(Timer::default())
    }
}

impl TickBehavior {
    pub fn static_timer() -> Self {
        Self::Static(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
    }
    pub fn individual() -> Self {
        Self::Individual(IndividualTargetTimers::default())
    }
}

#[derive(Default, Reflect, Debug, Clone)]
pub struct StaticTimer(pub Timer);

// map that holds what things have been hit by an ability instance
#[derive(Default, Debug, Clone, Reflect)]
pub struct IndividualTargetTimers {
    pub map: HashMap<Entity, Timer>,
}

pub struct FiringLifetime {
    pub ticks: Ticks,
    pub behavior: TickBehavior,
    pub interval: f32,
}

impl FiringLifetime {
    pub fn set_timer(&mut self) {
        match self.behavior {
            TickBehavior::Static(ref mut timer) => {
                timer.set_duration(Duration::from_millis(self.interval as u64));
            }
            _ => (),
        }
    }
    pub fn from_ticks(ticks: Ticks, lifetime: f32) -> Self {
        let i = match ticks {
            Ticks::Multiple(tick_num) => lifetime / tick_num as f32,
            Ticks::Once => lifetime,
            _ => 1.0,
        };
        FiringLifetime {
            ticks,
            behavior: TickBehavior::individual(),
            interval: i,
        }
    }
}

impl Default for FiringLifetime {
    fn default() -> Self {
        FiringLifetime::from_ticks(Ticks::Once, 2.0)
    }
}

#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct PausesWhenEmpty;

#[derive(Component, Debug, Clone, Default, Reflect)]
pub enum Ticks {
    #[default]
    Once,
    Multiple(u32),
    Unlimited,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct FiringInterval(pub f32);

impl Default for FiringInterval {
    fn default() -> Self {
        FiringInterval(1.0)
    }
}

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
    Homing(Ability), // All Abilities are Areas, but not all Areas are Abilities, Areas are parent of Abilities
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageType {
    Physical,
    Magical,
    True,
    Hybrid,
}

impl DamageType {
    pub fn get_color(&self) -> Color {
        match self {
            DamageType::Physical => Color::rgb(0.863, 0.4, 0.353),
            DamageType::True => Color::WHITE,
            DamageType::Magical => Color::rgb(0.569, 0.665, 0.943),
            DamageType::Hybrid => Color::PURPLE,
        }
    }
}
