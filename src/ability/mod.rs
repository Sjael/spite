use std::{collections::HashMap, time::Duration};

use crate::{
    actor::{
        buff::BuffInfo,
        crowd_control::{CCInfo, CCType},
        stats::Stat,
    },
    assets::Icons,
};
use bevy::prelude::*;
use derive_more::Display;
use leafwing_input_manager::Actionlike;

use self::{
    bundles::{BombInfo, DefaultAbilityInfo, FireballInfo, FrostboltInfo},
    shape::AbilityShape,
};

pub mod bundles;
pub mod rank;
pub mod shape;

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

pub struct AbilityBlueprint {
    pub base_ability: Ability,
    pub name: String,
    pub stages: HashMap<Trigger, AbilityStage>,
    pub cooldown: Cooldown,
}

pub struct AbilityStage {
    pub effects: HashMap<AbilityComp, RankNumbers>,
    pub shape: AbilityShape,
    pub path: Path,
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

pub enum AbilityComp {
    Scaling(Stat),
    BaseDamage,
    Cooldown,
    Cost,
    CC(CCType),
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
        let ability = FireballInfo::fire_bundle(&Transform::default());
        commands.spawn(ability).id()
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AbilityTooltip {
    pub title: String,
    pub image: String,
    pub description: String,
}

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
