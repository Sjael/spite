use bevy::{prelude::*, utils::HashMap};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
//use fixed::types::I40F24;
use crate::ability::HealthChangeEvent;
use crate::view::Spectating;
//use crate::buff::BuffMap;
use crate::{GameState};
use std::fmt::Display;

// Use enum as stat instead of unit structs?
//
//
#[derive(Reflect, Debug, Default, Clone, FromReflect, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Stat {
    #[default]
    Health,
    HealthRegen,
    HealthMax,
    CharacterResource,
    CharacterResourceRegen,
    CharacterResourceMax,
    Speed,
    Gold,
    Xp,
    PhysicalPower,
    Level,
}

impl Stat{
    fn get_base(self) -> f32{
        match self{
            Stat::Speed => 5.0,
            Stat::HealthMax => 465.0,
            Stat::HealthRegen => 5.0,
            Stat::CharacterResourceMax => 227.0,
            Stat::CharacterResourceRegen => 5.0,
            _ => 0.0,
        }
    }
}

impl Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
    //Scale,
    Max,
    Min,
}

impl Modifier {
    fn into_tag(self, stat: Stat) -> AttributeTag {
        AttributeTag::Modifier{
            modifier: self,
            target: Box::new(stat.into())
        }
    }
}
impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Vec<String>>();
        app.add_system(take_damage_or_heal.in_set(OnUpdate(GameState::InGame)));
        app.add_system(calculate_attributes.in_set(OnUpdate(GameState::InGame)));
        app.add_system(regen_health.in_set(OnUpdate(GameState::InGame)));
        app.add_system(regen_resource.in_set(OnUpdate(GameState::InGame)));
    }
}

pub fn take_damage_or_heal(
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

pub fn regen_health(
    mut query: Query<&mut Attributes>,
    time: Res<Time>,
){
    for mut attributes in query.iter_mut() {
        let healthregen = *attributes.get(&Stat::HealthRegen.into()).unwrap_or(&0.0);
        let health_max = *attributes.get(&Stat::HealthMax.into()).unwrap_or(&100.0);
        let health = attributes.entry(Stat::Health.into()).or_insert(1.0);
        if *health <= 0.0 {
            continue;
        }
        let mut result = *health + (healthregen * time.delta_seconds()) ;
        if result > health_max {
            result = health_max;
        }
        *health = result;
    }
}

pub fn regen_resource(
    mut query: Query<&mut Attributes>,
    time: Res<Time>,
){
    for mut attributes in query.iter_mut() {
        let regen = *attributes.get(&Stat::CharacterResourceRegen.into()).unwrap_or(&0.0);
        let max = *attributes.get(&Stat::CharacterResourceMax.into()).unwrap_or(&100.0);
        let resource = attributes.entry(Stat::CharacterResource.into()).or_default();
        let mut result = *resource + (regen * time.delta_seconds()) ;
        if result > max {
            result = max;
        }
        *resource = result;
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
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct Attributes(HashMap<AttributeTag, f32>);

impl Default for Attributes{
    fn default() -> Self {
        let mut map: HashMap<AttributeTag, f32> = HashMap::new();
        for stat in Stat::iter(){
            use Stat::*;
            match stat{
                Health | CharacterResource | Gold => continue, // these stats dont need multipliers, as they are temporal
                _ => ()
            }
            map.insert(AttributeTag::Modifier {
                modifier: Modifier::Base,
                target: Box::new(stat.clone().into()),
            }, stat.get_base());
        }
        Self(map)
    }
}

pub fn calculate_attributes(
    mut attributes: Query<(Entity, &mut Attributes), Changed<Attributes>>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    println!("first");
    for (entity, mut attributes) in &mut attributes {
        // sort by deepest modifier, so we process Mul<Add<Mul<Base<Health>>>> before Mul<Base<Health>>
        let mut tags = attributes.keys().cloned().collect::<Vec<_>>();
        tags.sort_by(|a, b| a.deepness().cmp(&b.deepness()));

        for tag in tags {
            match tag.clone() {
                AttributeTag::Modifier { modifier, target } => {
                    if entity == spectating{
                        println!("modifier: {} is at {}", tag, attributes.get(&tag).unwrap_or(&0.0).clone());
                    }
                    let modifier_attr = attributes.entry(tag).or_default().clone();
                    //let level = *attributes.clone().get(&Stat::Level.into()).unwrap_or(&1.0);\
                    let target_attr = attributes.entry(*target).or_default();

                    let modified = match modifier {
                        Modifier::Base => modifier_attr,
                        //Modifier::Scale => level * modifier_attr,
                        Modifier::Add => *target_attr + modifier_attr,
                        Modifier::Sub => *target_attr - modifier_attr,
                        Modifier::Mul => *target_attr * (1.0 + modifier_attr / 100.0),
                        Modifier::Div => *target_attr / (1.0 - modifier_attr / 100.0),
                        Modifier::Min => target_attr.max(modifier_attr),
                        Modifier::Max => target_attr.min(modifier_attr),
                    };

                    *target_attr = modified;
                }
                AttributeTag::Stat(_) => {
                    if entity == spectating{
                        println!("stat: {} is at {}", tag, attributes.get(&tag).unwrap_or(&4.04).clone());
                    }
                }
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



// only returning the modifier for the id, not the modifier and stat
// if an ability changes 2 different stats in the same way, add or mult, itll break probably (?)
impl Display for AttributeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self{
            AttributeTag::Modifier { modifier, target } => {
                write!(f, "{:?} {:?}", modifier, *target)
            }
            AttributeTag::Stat(stat) => write!(f, "{:?}", stat)
        }
    }
}

impl AttributeTag {
    pub fn ordering(&self) -> usize {
        match self {
            Self::Modifier { modifier, .. } => match modifier {
                // do these first
                Modifier::Base => 1,
                //Modifier::Scale => 2, 
                Modifier::Add => 3,
                Modifier::Sub => 3,

                // do these second
                Modifier::Mul => 10,
                Modifier::Div => 10,

                // Do these second to last
                Modifier::Max => 100,
                Modifier::Min => 100,
            },
            Self::Stat(..) => 1000,
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
