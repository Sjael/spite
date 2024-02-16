//! Construction of abilities.
//!
//! Basic premise of all abilities is there are `collectors`, `filters`, and `appliers`.
//!
//! `Collectors` use colliders/auto-targetters/etc. to determine potential targets for an ability.
//! `Filters` take collectors and disqualify them if they shouldn't be affected.
//! `Appliers` finalize the effects of the ability on the target entity, most commonly damaging or
//! debuffing.

use bevy::prelude::*;
use derive_more::Display;
use leafwing_input_manager::Actionlike;

use crate::{
    ability::{shape::AbilityShape, ticks::TickBehavior},
    buff::BuffInfo,
    crowd_control::CCInfo,
    prelude::*,
};

pub mod builder;
pub mod collector;
pub mod db;
pub mod shape;
pub mod ticks;

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

#[derive(Actionlike, Component, Reflect, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum Ability {
    Frostbolt,
    Fireball,
    Bomb,
    #[default]
    BasicAttack,
    Dash,
    BallistaShot,
}

impl Ability {
    pub fn get_shape_with_movement(&self) -> AbilityShape {
        let length = self.get_length();
        let lifetime = self.get_deployed_lifetime();
        let speed = self.get_speed();
        let width = self.get_shape().get_width();
        let length_with_movement = length + speed * lifetime;
        AbilityShape::Rectangle {
            length: length_with_movement,
            width: width,
        }
    }

    pub fn add_unique_components(&self, commands: &mut Commands, entity: Entity) {
        match self {
            Ability::Frostbolt => {
                commands.entity(entity).insert(MaxTargetsHit::new(1));
            }
            Ability::Fireball => {
                commands.entity(entity).insert(UniqueTargetsHit::default());
            }
            Ability::Bomb => {
                commands.entity(entity).insert(TickBehavior::new_individual(0.5));
            }
            Ability::BasicAttack => {
                commands
                    .entity(entity)
                    .insert((MaxTargetsHit::new(2), UniqueTargetsHit::default()));
            }
            Ability::Dash => {
                commands.entity(entity).insert(UniqueTargetsHit::default());
            }
            Ability::BallistaShot => {
                commands.entity(entity).insert(TargetFilter::closest(1));
            }
        }
    }

    pub fn hover(&self) -> impl Bundle {
        let length = self.get_length();
        let speed = self.get_speed();
        let lifetime = self.get_deployed_lifetime();

        let length_with_movement = length + speed * lifetime;

        let offset = if self.on_reticle() {
            Vec3::new(0.0, 0.0, -(length_with_movement - length) / 2.0)
        } else {
            Vec3::new(0.0, 0.0, -length_with_movement / 2.0)
        };
        (
            SpatialBundle::from_transform(Transform {
                translation: offset,
                ..default()
            }),
            Sensor,
            Targetter,
            self.get_shape_with_movement(),
            *self,
        )
    }
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsInArea {
    pub list: Vec<Entity>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsHittable {
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct TargetFilter {
    pub target_selection: TargetSelection,
    pub number_of_targets: u8,
    pub list: Vec<Entity>,
}

impl TargetFilter {
    fn closest(num: u8) -> Self {
        Self {
            number_of_targets: num,
            target_selection: TargetSelection::Closest,
            ..default()
        }
    }
}

#[derive(Default, Debug, Clone, Reflect)]
pub enum TargetSelection {
    #[default]
    Closest,
    Random,
    HighestThreat, // Make Threat hashmap of <(Entity, Threat)>
    All,
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

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct UniqueTargetsHit {
    pub already_hit: Vec<Entity>,
}

#[derive(Component, Default, Deref, Clone, Debug)]
pub struct Tags(pub Vec<TagInfo>);

#[derive(Clone, Debug)]
pub enum TagInfo {
    Heal(f32),
    Damage(f32),
    Buff(BuffInfo),
    CC(CCInfo),
    Homing(Ability), // Once an Ability turns into a spawned entity, it is an Area, Abilities are simply blueprints rn
    ResourcePerTarget(i32),
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

#[derive(Component)]
pub struct Targetter;
