use bevy::{prelude::*, utils::HashMap};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use crate::ability::{Ability, DamageType};
//use fixed::types::I40F24;
use crate::area::HealthChangeEvent;
//use crate::buff::BuffMap;
use crate::GameState;
use crate::game_manager::InGameSet;
use std::fmt::Display;
use std::time::Instant;

// Use enum as stat instead of unit structs?
//
//
#[derive(Reflect, Debug, Default, Clone , PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
pub enum Stat {
    Xp,
    Level,
    Speed,
    #[default]
    Health,
    HealthRegen,
    HealthMax,
    CharacterResource,
    CharacterResourceRegen,
    CharacterResourceMax,
    Gold,
    PhysicalPower,
    MagicalPower,
    PhysicalProtection,
    MagicalProtection,
    PhysicalPenetration,
    MagicalPenetration,
    AttacksPerSecond,
    CooldownReduction,
}

impl Stat{
    fn get_base(self) -> f32{
        use Stat::*;
        match self{
            Level => 1.0,
            Speed => 5.0,
            HealthMax => 465.0,
            HealthRegen => 12.0,
            CharacterResourceMax => 227.0,
            CharacterResourceRegen => 5.0,
            MagicalProtection => 55.0,
            PhysicalProtection => 25.0,
            MagicalPower => 45.0,
            PhysicalPower => 200.0,
            CooldownReduction => 50.0,
            _ => 0.0,
        }
    }
}

impl Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Stat{
    pub fn as_tag(self) -> AttributeTag{
        AttributeTag::from(self)
    }
}



impl From<Stat> for AttributeTag{
    fn from(src: Stat) -> AttributeTag{
        AttributeTag::Stat(src)
    }
}

#[derive(Reflect, Debug, Default, Clone , PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn into_tag(self, stat: Stat) -> AttributeTag {
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
        app.add_event::<HealthMitigatedEvent>();
        app.register_type::<Vec<String>>();
        app.add_systems(Update, (
            calculate_attributes,
            regen_health,
            regen_resource,
            calculate_health_change,
            apply_health_change,
        ).chain().in_set(InGameSet::Update));
    }
}

#[derive(Event, Clone)]
pub struct HealthMitigatedEvent{
    pub change: i32,
    pub mitigated: u32,
    pub ability: Ability,
    pub attacker: Entity,
    pub defender: Entity,
    pub sensor: Entity,
    pub damage_type: DamageType,
    pub when: Instant,
}

pub fn calculate_health_change(
    mut health_events: EventReader<HealthChangeEvent>,
    mut health_mitigated_events: EventWriter<HealthMitigatedEvent>,
    attribute_query: Query<&Attributes>,
) {
    for event in health_events.iter() {
        let Ok(defender_stats) = attribute_query.get(event.defender) else {continue};
        let attacker_stats = if let Ok(attacker_stats) = attribute_query.get(event.attacker){
            attacker_stats.clone()
        } else {
            Attributes::default() // can prob optimize this later
        };

        let (prots, pen) = if event.damage_type == DamageType::Physical {
            (
                *defender_stats.get(&Stat::PhysicalProtection.as_tag()).unwrap_or(&100.0),
                *attacker_stats.get(&Stat::PhysicalPenetration.as_tag()).unwrap_or(&0.0)
            )

        } else if event.damage_type == DamageType::Magical {
            (
                *defender_stats.get(&Stat::MagicalProtection.as_tag()).unwrap_or(&100.0),
                *attacker_stats.get(&Stat::MagicalPenetration.as_tag()).unwrap_or(&0.0)
            )
        } else {
            (0.0 , 0.0)
        };
        let percent_pen = 10.0;
        let prots_penned = prots * percent_pen / 100.0 + pen;
        let post_mitigation_damage = (event.amount * 100.0 / (100.0 + prots - prots_penned)).ceil() as i32; 
        // ceil = round up, so damage gets -1 and healing gets +1, might use floor to nerf healing if op LOL
        let mitigated = (event.amount as i32 - post_mitigation_damage).abs() as u32;
        health_mitigated_events.send(HealthMitigatedEvent{
            change: post_mitigation_damage,
            mitigated: mitigated,
            ability: event.ability,
            attacker: event.attacker,
            defender: event.defender,
            sensor: event.sensor,
            damage_type: event.damage_type,
            when: event.when,
        });
    }
}

pub fn apply_health_change(
    mut health_mitigated_events: EventReader<HealthMitigatedEvent>,
    mut health_query: Query<&mut Attributes>,
){
    for event in health_mitigated_events.iter() {
        let Ok(mut defender_stats) = health_query.get_mut(event.defender) else {continue};
        let health = defender_stats.entry(Stat::Health.as_tag()).or_default();
        /*
        if event.change > 0 {
            println!("healing is {:?}", event.change);
        } else {
            println!("damage is {:?}", event.change);
        }
         */
        let new_hp = *health + event.change as f32; // Add since we flipped number back in team detection
        *health = new_hp;
    }
}
pub fn regen_health(
    mut query: Query<&mut Attributes>,
    time: Res<Time>,
){
    for mut attributes in query.iter_mut() {
        let healthregen = *attributes.get(&Stat::HealthRegen.as_tag()).unwrap_or(&0.0);
        let health_max = *attributes.get(&Stat::HealthMax.as_tag()).unwrap_or(&100.0);
        let health = attributes.entry(Stat::Health.as_tag()).or_insert(1.0);
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
        let regen = *attributes.get(&Stat::CharacterResourceRegen.as_tag()).unwrap_or(&0.0);
        let max = *attributes.get(&Stat::CharacterResourceMax.as_tag()).unwrap_or(&100.0);
        let resource = attributes.entry(Stat::CharacterResource.as_tag()).or_default();
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
                Health | CharacterResource | Gold => continue, // these stats dont need modifier stack, add xp and level?
                _ => ()
            }
            map.insert(
                AttributeTag::Modifier {
                    modifier: Modifier::Base,
                    target: Box::new(stat.clone().into()),
                }, 
                stat.get_base()
            );
        }
        Self(map)
    }
}

pub fn calculate_attributes(
    mut attributes: Query<&mut Attributes, Changed<Attributes>>,
){
    for mut attributes in &mut attributes {
        // sort by deepest modifier, so we process Mul<Add<Mul<Base<Health>>>> before Mul<Base<Health>>
        let mut tags = attributes.keys().cloned().collect::<Vec<_>>();
        tags.sort_by(|a, b| a.ordering().cmp(&b.ordering()));

        for tag in tags {
            match tag.clone() {
                AttributeTag::Modifier { modifier, target } => {
                    let modifier_attr = attributes.entry(tag).or_default().clone();
                    //let level = *attributes.clone().get(&Stat::Level.into()).unwrap_or(&1.0);\
                    let target_attr = attributes.entry(*target).or_default();

                    let modified = match modifier {
                        Modifier::Base => modifier_attr,
                        //Modifier::Scale => *target_attr + level * modifier_attr,
                        Modifier::Add => *target_attr + modifier_attr,
                        Modifier::Sub => *target_attr - modifier_attr,
                        Modifier::Mul => *target_attr * (1.0 + modifier_attr / 100.0),
                        Modifier::Div => *target_attr / modifier_attr,
                        Modifier::Min => target_attr.max(modifier_attr),
                        Modifier::Max => target_attr.min(modifier_attr),
                    };

                    *target_attr = modified;
                }
                AttributeTag::Stat(_) => ()
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Reflect )]
pub enum AttributeTag {
    Modifier {
        modifier: Modifier,
        #[reflect(ignore)]
        target: Box<AttributeTag>,
    },
    Stat(Stat),
}

impl Default for AttributeTag{
    fn default() -> Self {
        Self::Stat(Stat::Health)
    }
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
                Modifier::Sub => 3, // Move to after div for reduction of stats?

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
}