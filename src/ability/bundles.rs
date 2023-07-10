use crate::{
    buff::{BuffInfo, BuffTargets, BuffType},
    crowd_control::{CCInfo, CCType},
    stats::{Attributes, Stat}, area::DamageType,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{ActiveEvents, RigidBody, Sensor, Velocity};

use super::{*, shape::AbilityShape};

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

pub trait AbilityBehavior{

    fn spawn(&self) -> Entity;
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
            MaxTargetsHit::new(1),
            Tags {
                list: vec![
                    TagInfo::Damage(38.0),
                    TagInfo::CC(CCInfo {
                        cctype: CCType::Stun,
                        duration: 1.0,
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

    pub fn hover_bundle() -> impl Bundle{
        let length = 0.8;
        let width= 0.5;

        let speed = 18.0;
        let lifetime = 1.0;

        let length_with_movement = length + speed * lifetime;
        let fires_from_reticle = false;

        
        let offset;
        if fires_from_reticle {
            offset = Vec3::new(0.0, 0.0, -(length_with_movement - length) / 2.0 );
        } else {
            offset = Vec3::new(0.0, 0.0, -length_with_movement / 2.0 );
        }
        (           
        AbilityShape::Rectangle {
            length: length_with_movement,
            width: width,
        },
        SpatialBundle::from_transform(Transform {
            translation: offset,
            ..default()
        }),
        Sensor,
        Targetter,
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
        let speed = 22.0;
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
        DamageType::Magical,
        UniqueTargetsHit::default(),
        CastingLifetime { seconds: 2.0 },
        Tags {
            list: vec![TagInfo::Damage(11.0)],
        },
    )}

    pub fn hover_bundle() -> impl Bundle{
        let radius = 1.0;
        let angle= 360.;

        let length = radius * 2.0;

        let speed = 22.0;
        let lifetime = 2.0;
        let length_with_movement = length + speed * lifetime;
        let fires_from_reticle = true;

        let offset;
        if fires_from_reticle {
            offset = Vec3::new(0.0, 0.0, -(length_with_movement - length) / 2.0 );
        } else {
            offset = Vec3::new(0.0, 0.0, -length_with_movement / 2.0 );
        }
        (           
        AbilityShape::Rectangle {
            length: length_with_movement,
            width: radius * 2.0,
        },
        SpatialBundle::from_transform(Transform {
            translation: offset,
            ..default()
        }),
        Sensor,
        Targetter,
    )}
}

pub struct BombInfo;
// Make this an Ability trait ?
impl BombInfo {
    pub fn fire_bundle(transform: &Transform) -> impl Bundle {
        let direction = transform.rotation * -Vec3::Z;
        let speed = 2.0;
        (
        Name::new("Bomb"),
        Ability::Bomb,
        AbilityShape::Arc {
            radius: 1.5,
            angle: 360.,
        },
        SpatialBundle::from_transform(transform.clone()),
        RigidBody::KinematicVelocityBased,
        Velocity {
            linvel: direction * speed,
            ..default()
        },
        Sensor,
        Ticks::Unlimited,
        DamageType::Magical,
        FiringInterval(200),
        TickBehavior::individual(),
        CastingLifetime { seconds: 2.0 },
        Tags {
            list: vec![TagInfo::Damage(16.0)],
        },
    )}

    pub fn hover_bundle() -> impl Bundle{
        let radius = 1.5;
        let angle= 360.;

        let length = radius * 2.0;

        let speed = 2.0;
        let lifetime = 1.0;
        let length_with_movement = length + speed * lifetime;
        let fires_from_reticle = true;

        let offset;
        if fires_from_reticle {
            offset = Vec3::new(0.0, 0.0, -(length_with_movement - length) / 2.0 );
        } else {
            offset = Vec3::new(0.0, 0.0, -length_with_movement / 2.0 );
        }
        (           
        AbilityShape::Arc {
            radius: radius,
            angle: angle,
        },
        SpatialBundle::from_transform(Transform {
            translation: offset,
            ..default()
        }),
        Sensor,
        Targetter,
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
        let speed = 30.0;
        (
        Name::new("DefaultAbility"),
        Ability::BasicAttack,
        AbilityShape::default(),
        SpatialBundle::from_transform(transform.clone()),
        DamageType::Physical,
        RigidBody::KinematicVelocityBased,
        Velocity {
            linvel: direction * speed,
            ..default()
        },
        Sensor,
        MaxTargetsHit::new(1),
        CastingLifetime { seconds: 2.0 },
        Tags {
            list: vec![TagInfo::Damage(11.0)],
        },
    )}


    pub fn hover_bundle() -> impl Bundle{
        let length = 2.0;
        let width= 3.0;

        let speed = 30.0;
        let lifetime = 2.0;

        let length_with_movement = length + speed * lifetime;
        let fires_from_reticle = false;
        
        let offset;
        if fires_from_reticle {
            offset = Vec3::new(0.0, 0.0, -(length_with_movement - length) / 2.0 );
        } else {
            offset = Vec3::new(0.0, 0.0, -length_with_movement / 2.0 );
        }
        (           
        AbilityShape::Rectangle {
            length: length_with_movement,
            width: width,
        },
        SpatialBundle::from_transform(Transform {
            translation: offset,
            ..default()
        }),
        Sensor,
        Targetter,
    )}
}
