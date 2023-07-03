use crate::{
    buff::{BuffInfo, BuffTargets, BuffType},
    crowd_control::{CCInfo, CCType},
    stats::{Attributes, Stat},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{ActiveEvents, RigidBody, Sensor, Velocity};

use super::*;

#[derive(Component)]
pub struct Caster(pub Entity);

#[derive(Component)]
pub struct Targetter;

pub enum AbilityInfo{
    Frostbolt(FrostboltInfo),
    Fireball(FireballInfo),
    Shadowbolt(DefaultAbilityInfo),
}

impl AbilityInfo{
    pub fn get_info(ability: &Ability) -> Self{
        use AbilityInfo::*;
        match ability{
            Ability::Frostbolt => Frostbolt(FrostboltInfo::default()),
            Ability::Fireball => Fireball(FireballInfo::default()),
            _ => Shadowbolt(DefaultAbilityInfo::default()),
        }
    }
}


#[derive(Bundle, Clone, Debug)]
pub struct SpatialAbilityBundle {
    pub id: Ability,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub shape: AbilityShape,
    pub sensor: Sensor,
    pub events: ActiveEvents,
    pub tick_behavior: TickBehavior,
    pub targetsinarea: TargetsInArea,
    pub targetstoeffect: TargetsHittable,
    pub tags: Tags,
    pub lifetime: CastingLifetime,
}

impl Default for SpatialAbilityBundle {
    fn default() -> Self {
        Self {
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
    pub fn fire_bundle(transform: &Transform) -> impl Bundle{        
        let direction = transform.rotation * -Vec3::Z;
        let speed = 18.0;
        (
            Name::new("Frostbolt"),
            Ability::Frostbolt,
            AbilityShape::Rectangle {
                length: 0.8,
                width: 0.5,
            },
            SpatialBundle::from_transform(transform.clone()),
            Velocity {
                linvel: direction * speed,
                ..default()
            },
            RigidBody::KinematicVelocityBased,
            Sensor,
            CastingLifetime { seconds: 1.0 },
            Tags {
                list: vec![
                    TagInfo::Damage(44.0),
                    TagInfo::CC(CCInfo {
                        cctype: CCType::Stun,
                        duration: 20.0,
                    }),
                    TagInfo::Buff(BuffInfo {
                        stat: Stat::Health.into(),
                        amount: 10.0,
                        duration: 10.0,
                        ..default()
                    }),
                ],
            },
    )}

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
    pub fn fire_bundle(transform: &Transform) -> impl Bundle {
        let direction = transform.rotation * -Vec3::Z;
        let speed = 20.0;
        (
        Name::new("Fireball"),
        Ability::Fireball,
        AbilityShape::Arc {
            radius: 1.,
            angle: 360.,
        },
        SpatialBundle::from_transform(transform.clone()),
        RigidBody::KinematicVelocityBased,
        Velocity {
            linvel: direction * speed,
            ..default()
        },
        Sensor,
        CastingLifetime { seconds: 5.0 },
        Tags {
            list: vec![TagInfo::Damage(11.0)],
        },
    )}

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
    pub fn fire_bundle(transform: &Transform) -> impl Bundle {
        let direction = transform.rotation * -Vec3::Z;
        let speed = 20.0;
        (
        Name::new("DefaultAbility"),
        Ability::BasicAttack,
        AbilityShape::default(),
        SpatialBundle::from_transform(transform.clone()),
        RigidBody::KinematicVelocityBased,
        Velocity {
            linvel: direction * speed,
            ..default()
        },
        Sensor,
        CastingLifetime { seconds: 5.0 },
        Tags {
            list: vec![TagInfo::Damage(11.0)],
        },
    )}


}
