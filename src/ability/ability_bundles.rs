
use bevy::prelude::*;
use crate::{crowd_control::{CCType, CCInfo}, stats::Stat, buff::BuffInfoTest, game_manager::TEAM_ALL};
use bevy_rapier3d::prelude::{Velocity, RigidBody, Sensor, ActiveEvents};

use super::*;





#[derive(Bundle, Clone, Debug)]
pub struct SpatialAbilityBundle{
    pub id: Ability,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub shape: AbilityShape,
    pub sensor: Sensor,
    pub events: ActiveEvents,
    pub effectapplytype: EffectApplyType,
    pub targetsinarea: TargetsInArea,
    pub targetstoeffect: TargetsToEffect,
    pub tags: Tags,
    pub lifetime: CastingLifetime,
}

impl Default for SpatialAbilityBundle{
    fn default() -> Self {
        Self{
            id: Ability::BasicAttack,
            name: Name::new("Basic Attack"),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            ..default()
        }
    }
}

//
// Frostbolt
//

pub struct FrostboltInfo {
    name: Name,
    id: Ability,

    shape: AbilityShape,
    pub transform: Transform,
}

impl Default for FrostboltInfo {
    fn default() -> Self {
        Self {
            name: Name::new("Frostbolt"),
            id: Ability::Frostbolt,
            shape: AbilityShape::Rectangle {
                length: 0.8,
                width: 0.5,
            },
            transform: Transform::default(),
        }
    }
}

// Make this an Ability trait ?
impl FrostboltInfo {

    pub fn fire(&self, commands: &mut Commands, transform: &Transform) -> Entity {
        let direction = transform.rotation * -Vec3::Z;
        commands.spawn((
            self.name.clone(),
            self.id.clone(),
            self.shape.clone(),
            transform.clone(),
            GlobalTransform::default(),
            Velocity {
                linvel: direction * 20.0,
                ..default()
            },
            RigidBody::KinematicVelocityBased,
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            CastingLifetime { seconds: 1.0 },
            EffectApplyType::default(),
            TargetsInArea::default(),
            TargetsToEffect::default(),
            Tags{
                list: vec![
                    TagInfo::Damage(44.0),
                    TagInfo::CC(CCInfo{
                        cctype: CCType::Stun,
                        duration: 20.0,
                    }),
                    TagInfo::Buff(BuffInfoTest{
                        stat: Stat::Health,
                        duration: 20.0,
                    }),
                ]
            },
        )).id()
    }
}

//
// Fireball
//

pub struct FireballInfo {
    name: Name,
    id: Ability,

    shape: AbilityShape,
    pub transform: Transform,
}

impl Default for FireballInfo {
    fn default() -> Self {
        Self {
            name: Name::new("Fireball"),
            id: Ability::Fireball,
            shape: AbilityShape::Arc {
                radius: 1.,
                angle: 360.,
            },
            transform: Transform::default(),
        }
    }
}

// Make this an Ability trait ?
impl FireballInfo {

    pub fn fire(&self, commands: &mut Commands, transform: &Transform) -> Entity {
        let direction = transform.rotation * -Vec3::Z;
        commands.spawn((
            self.name.clone(),
            self.id.clone(),
            self.shape.clone(),
            transform.clone(),
            GlobalTransform::default(),
            RigidBody::KinematicVelocityBased,
            Velocity {
                linvel: direction * 20.0,
                ..default()
            },
            Sensor,
            CastingLifetime { seconds: 5.0 },
        )).id()
    }
}

//
// Default
//

pub struct DefaultAbilityInfo {
    name: Name,
    id: Ability,
    shape: AbilityShape,
    pub transform: Transform,
}

impl Default for DefaultAbilityInfo {
    fn default() -> Self {
        Self {
            name: Name::new("DefaultAbility"),
            id: Ability::BasicAttack,
            shape: AbilityShape::default(),
            transform: Transform::default(),
        }
    }
}


#[derive(Component)]
pub struct FloatingDamage(pub u32);

// Make this an Ability trait ?
impl DefaultAbilityInfo {

    pub fn fire(&self, commands: &mut Commands, transform: &Transform) -> Entity {
        let direction = transform.rotation * -Vec3::Z;
        commands.spawn((
            self.name.clone(),
            self.id.clone(),
            self.shape.clone(),
            transform.clone(),
            GlobalTransform::default(),
            Velocity {
                linvel: direction * 10.0,
                ..default()
            },
            RigidBody::KinematicVelocityBased,
            Sensor,
            CastingLifetime { seconds: 1.0 },
            FloatingDamage(32),
        )).id()
    }
}