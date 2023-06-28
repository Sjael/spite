use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, ops::MulAssign};
//use fixed::types::I40F24;
use crate::ability::HealthChangeEvent;
//use crate::buff::BuffMap;
use crate::{on_gametick, GameState};
use std::ops::{Add, AddAssign, Deref, DerefMut, Mul};

// Use enum as stat instead of unit structs?
//
//
#[derive(Reflect, Debug, Default, Clone, FromReflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stat {
    #[default]
    Health,
    Speed,
    Gold,
    Xp,
    PhysicalPower,
    CharacterResource,
}

impl Into<AttributeTag> for Stat {
    fn into(self) -> AttributeTag {
        AttributeTag::Stat(self)
    }
}

#[derive(Reflect, Debug, Default, Clone, FromReflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Modifier {
    #[default]
    Add,
    Mul,
    Sub,
    Div,
    Base,
    Max,
    Min,
}

impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Reflect, Debug, FromReflect)]
pub struct Health;
#[derive(Reflect, Debug, FromReflect)]
pub struct CharacterResource;
#[derive(Reflect, Debug)]
pub struct Gold;
#[derive(Reflect, Debug)]
pub struct Experience;
#[derive(Reflect, Debug)]
pub struct MovementSpeed;
#[derive(Reflect, Debug)]
pub struct PhysicalPower;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Vec<String>>();
        app.add_system(change_health.in_set(OnUpdate(GameState::InGame)));
        app.add_system(calculate_attributes.in_set(OnUpdate(GameState::InGame)));
    }
}

pub fn change_health(
    mut health_events: EventReader<HealthChangeEvent>,
    mut health_query: Query<&mut Attributes>,
) {
    for event in health_events.iter() {
        let Ok(mut attributes) = health_query.get_mut(event.defender) else {continue};
        let health = attributes.entry(Stat::Health.into()).or_default();

        if event.amount > 0.0 {
            //println!("healing is {:?}", event.amount);
        } else {
            //println!("damage is {:?}", event.amount);
        }
        let new_hp = *health + event.amount; // Add since we flipped number back in team detection
        *health = new_hp;
    }
}

/*
basic case:
Mul<Base<Health>> = 1.1;
Base<Health> = 110.0;
Health = 110.0;

regen case:
Add<Health> = 1.0;
Health = 50.0;
Max<Health> = 100.0;

fetch Add<Health>, fetch Health
Health = 50.0 + 1.0;
fetch Max<Health>, fetch Health
Health = 51.0.max(100.0) == 51.0;
 */
#[derive(Component, Debug, Default, Clone, Deref, DerefMut)]
pub struct Attributes(HashMap<AttributeTag, f32>);

pub fn calculate_attributes(mut attributes: Query<&mut Attributes>) {
    for mut attributes in &mut attributes {
        // sort by deepest modifier, so we process Mul<Add<Mul<Base<Health>>>> before Mul<Base<Health>>
        let mut tags = attributes.keys().cloned().collect::<Vec<_>>();
        tags.sort_by(|a, b| a.deepness().cmp(&b.deepness()));

        for tag in tags {
            match tag.clone() {
                AttributeTag::Modifier { modifier, target } => {
                    let modifier_attr = attributes.entry(tag).or_default().clone();

                    let target_attr = attributes.entry(*target).or_default();

                    let modified = match modifier {
                        Modifier::Add => *target_attr + modifier_attr,
                        Modifier::Mul => *target_attr * modifier_attr,
                        Modifier::Sub => *target_attr - modifier_attr,
                        Modifier::Div => *target_attr / modifier_attr,
                        Modifier::Base => *target_attr,
                        Modifier::Min => target_attr.max(modifier_attr),
                        Modifier::Max => target_attr.min(modifier_attr),
                    };

                    *target_attr = modified;
                }
                AttributeTag::Stat(stat) => {}
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum AttributeTag {
    Modifier {
        modifier: Modifier,
        target: Box<AttributeTag>,
    },
    Stat(Stat),
}

impl AttributeTag {
    pub fn ordering(&self) -> usize {
        match self {
            Self::Modifier { modifier, .. } => match modifier {
                // do these first
                Modifier::Add => 1,
                Modifier::Sub => 1,

                // do these second
                Modifier::Mul => 10,
                Modifier::Div => 10,

                // Do these second to last
                Modifier::Max => 100,
                Modifier::Min => 100,

                // Do these last
                Modifier::Base => 1000,
            },
            Self::Stat(..) => 0,
        }
    }

    pub fn deepness(&self) -> usize {
        self.deepness_from(0)
    }

    pub fn deepness_from(&self, mut current: usize) -> usize {
        match self {
            Self::Modifier { target, .. } => {
                current += 1;
                target.deepness_from(current)
            }
            Self::Stat(..) => current,
        }
    }
}
