use std::{fmt::Display, time::Instant};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

//use fixed::types::I40F24;
use crate::{
    ability::{Ability, DamageType},
    actor::player::Player,
    area::queue::HealthChangeEvent,
    prelude::{ActorState, Icons},
    previous::previous,
    session::director::InGameSet,
};

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthMitigatedEvent>();

        app.register_type::<Vec<String>>();
        app.register_type::<Attributes>();
        app.register_type::<HashSet<AttributeTag>>();
        app.register_type::<Stat>()
            .register_type::<Modifier>()
            .register_type::<AttributeTag>();

        app.add_systems(
            PreUpdate,
            (
                (
                    calculate_attributes,
                    regen_health,
                    regen_resource,
                    calculate_health_change,
                    apply_health_change,
                )
                    .chain(),
                spool_gold,
            )
                .in_set(InGameSet::Pre),
        );

        app.add_systems(Last, previous::<Attributes>);
    }
}

fn calculate_attributes(mut attributes: Query<&mut Attributes, Changed<Attributes>>) {
    for mut attributes in &mut attributes {
        let mut tags = attributes.attrs.keys().cloned().collect::<Vec<_>>();
        tags.sort_by(|a, b| a.ordering().cmp(&b.ordering()));

        for tag in tags {
            match tag.clone() {
                AttributeTag::Modifier { modifier, target } => {
                    let modifier_attr = attributes.get(tag);
                    //let level = *attributes.clone().get(&Stat::Level.into()).unwrap_or(&1.0);\
                    let old = attributes.get_mut(*target);

                    let new = match modifier {
                        Modifier::Base => modifier_attr,
                        //Modifier::Scale => *old + level * modifier_attr,
                        Modifier::Add => *old + modifier_attr,
                        Modifier::Sub => *old - modifier_attr,
                        Modifier::Mul => *old * (modifier_attr + 100.0) / 100.0,
                        Modifier::Div => *old / (modifier_attr + 100.0) * 100.0,
                        Modifier::Min => old.max(modifier_attr),
                        Modifier::Max => old.min(modifier_attr),
                    };
                    *old = new;
                }
                AttributeTag::Stat(_) => (),
            }
        }
    }
}

fn regen_health(mut query: Query<&mut Attributes>, time: Res<Time>) {
    for mut attributes in query.iter_mut() {
        let regen = attributes.get(Stat::HealthRegen);
        let max = attributes.get(Stat::HealthMax);
        let health = attributes.get_mut(Stat::Health);
        if *health <= 0.0 {
            continue
        }
        let result = *health + (regen * time.delta_seconds());
        *health = result.clamp(0.0, max);
    }
}

fn regen_resource(mut query: Query<&mut Attributes>, time: Res<Time>) {
    for mut attributes in query.iter_mut() {
        let regen = attributes.get(Stat::CharacterResourceRegen);
        let max = attributes.get(Stat::CharacterResourceMax);
        let resource = attributes.get_mut(Stat::CharacterResource);
        let result = *resource + (regen * time.delta_seconds());
        *resource = result.clamp(0.0, max);
    }
}

fn spool_gold(mut attribute_query: Query<&mut Attributes, With<Player>>, time: Res<Time>) {
    let gold_per_second = 3.0;
    for mut attributes in attribute_query.iter_mut() {
        let gold = attributes.get_mut(Stat::Gold);
        *gold += gold_per_second * time.delta_seconds();
    }
}

fn calculate_health_change(
    mut health_events: EventReader<HealthChangeEvent>,
    mut health_mitigated_events: EventWriter<HealthMitigatedEvent>,
    attribute_query: Query<&Attributes>,
) {
    for event in health_events.read() {
        let Ok(defender_stats) = attribute_query.get(event.defender) else { continue };
        let attacker_stats = if let Ok(attacker_stats) = attribute_query.get(event.attacker) {
            attacker_stats.clone()
        } else {
            Attributes::default() // can prob optimize this later
        };

        let (prots, pen) = if event.damage_type == DamageType::Physical {
            (
                defender_stats.get(Stat::PhysicalProtection),
                attacker_stats.get(Stat::PhysicalPenetration),
            )
        } else if event.damage_type == DamageType::Magical {
            (
                defender_stats.get(Stat::MagicalProtection),
                attacker_stats.get(Stat::MagicalPenetration),
            )
        } else {
            (0.0, 0.0)
        };
        let percent_pen = 10.0;
        let prots_penned = prots * percent_pen / 100.0 + pen;
        let post_mitigation_damage = (event.amount * 100.0 / (100.0 + prots - prots_penned)).ceil() as i32;
        // ceil = round up, so damage gets -1 and healing gets +1, might use floor to
        // nerf healing if op LOL
        let mitigated = (event.amount as i32 - post_mitigation_damage).abs() as u32;
        health_mitigated_events.send(HealthMitigatedEvent {
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

fn apply_health_change(
    mut health_mitigated_events: EventReader<HealthMitigatedEvent>,
    mut health_query: Query<(&mut ActorState, &mut Attributes)>,
) {
    for event in health_mitigated_events.read() {
        let Ok((mut actor_state, mut defender_stats)) = health_query.get_mut(event.defender) else { continue };
        let health = defender_stats.get_mut(Stat::Health);
        /*
        if event.change > 0 {
            println!("healing is {:?}", event.change);
        } else {
            println!("damage is {:?}", event.change);
        }
         */
        let new_hp = *health + event.change as f32; // Add since we flipped number back in team detection
        *health = new_hp;
        if new_hp < 0.0 {
            *actor_state = ActorState::Dead;
        }
    }
}

// Stats that the character has in the bottom left
// Let be customizable later
lazy_static! {
    pub static ref LISTED_STATS: Vec<Stat> = vec![
        Stat::Gold,
        Stat::PhysicalPower,
        Stat::PhysicalPenetration,
        Stat::Speed,
        Stat::CharacterResourceMax
    ];
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
#[reflect(Debug, PartialEq)]
pub enum Stat {
    // Temporal
    Xp,
    Level,
    Gold,
    Health,
    CharacterResource,

    // modifiers
    HealthRegen,
    HealthMax,
    CharacterResourceRegen,
    CharacterResourceMax,

    //
    Speed,
    PhysicalPower,
    MagicalPower,
    PhysicalProtection,
    MagicalProtection,
    PhysicalPenetration,
    MagicalPenetration,
    AttacksPerSecond,
    CooldownReduction,
}

impl Stat {
    fn get_base(self) -> f32 {
        use Stat::*;
        match self {
            Level => 1.0,
            Speed => 5.0,
            HealthMax => 465.0,
            HealthRegen => 5.0,
            CharacterResourceMax => 4.0,
            CharacterResourceRegen => 0.1,
            MagicalProtection => 55.0,
            PhysicalProtection => 25.0,
            MagicalPower => 45.0,
            PhysicalPower => 200.0,
            CooldownReduction => 50.0,
            _ => 0.0,
        }
    }
    pub fn get_color(self) -> Color {
        use Stat::*;
        match self {
            Gold => Color::YELLOW,
            _ => Color::WHITE,
        }
    }
    pub fn get_icon(self, icons: &Res<Icons>) -> Handle<Image> {
        use Stat::*;
        let image = match self {
            Speed => &icons.frostbolt,
            _ => &icons.basic_attack,
        };
        image.clone().into()
    }

    pub fn add(self) -> AttributeTag {
        AttributeTag::Modifier {
            modifier: Modifier::Add,
            target: Box::new(self.into()),
        }
    }
    pub fn mult(self) -> AttributeTag {
        AttributeTag::Modifier {
            modifier: Modifier::Mul,
            target: Box::new(self.into()),
        }
    }
}

impl Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = match self {
            Stat::Xp => "Experience",
            Stat::Speed => "Movement Speed",
            Stat::HealthRegen => "Health Regen",
            Stat::HealthMax => "Max Health",
            Stat::CharacterResource => "Resource",
            Stat::CharacterResourceRegen => "Resource Regen",
            Stat::CharacterResourceMax => "Max Resource",
            Stat::PhysicalPower => "Physical Power",
            Stat::MagicalPower => "Magical Power",
            Stat::PhysicalProtection => "Armor",
            Stat::MagicalProtection => "Spell Shield",
            Stat::PhysicalPenetration => "Armor Penetration",
            Stat::MagicalPenetration => "Spell Pierce",
            Stat::AttacksPerSecond => "Attack Speed",
            Stat::CooldownReduction => "Cooldown Reduction",
            Stat::Level => "Level",
            Stat::Health => "Health",
            Stat::Gold => "Gold",
        };
        write!(f, "{}", string)
    }
}

#[derive(Reflect, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[reflect(Debug, Default, PartialEq)]
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

impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Debug, Default, PartialEq)]
pub enum AttributeTag {
    Modifier {
        modifier: Modifier,
        #[reflect(ignore)]
        target: Box<AttributeTag>,
    },
    Stat(Stat),
}

impl From<Stat> for AttributeTag {
    fn from(src: Stat) -> AttributeTag {
        AttributeTag::Stat(src)
    }
}

impl Default for AttributeTag {
    fn default() -> Self {
        Stat::Health.into()
    }
}

impl std::fmt::Debug for AttributeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AttributeTag::Modifier { modifier, target } => {
                write!(f, "{:?} {:?}", modifier, *target)
            }
            AttributeTag::Stat(stat) => write!(f, "{:?}", stat),
        }
    }
}

// only returning the modifier for the id, not the modifier and stat
// if an ability changes 2 different stats in the same way, add or mult, itll
// break probably (?)
impl Display for AttributeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AttributeTag::Modifier { modifier, target } => {
                write!(f, "{:?} {:?}", modifier, *target)
            }
            AttributeTag::Stat(stat) => write!(f, "{:?}", stat),
        }
    }
}

impl AttributeTag {
    pub fn ordering(&self) -> usize {
        match self {
            Self::Modifier { modifier, .. } => match modifier {
                // do these first
                Modifier::Base => 1,
                //Modifier::Scale => 2, // stats you get per level
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
    pub fn target_stat(&self) -> Stat {
        match self {
            AttributeTag::Modifier { target, .. } => target.target_stat(),
            AttributeTag::Stat(stat) => *stat,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Attributes {
    dirty: HashSet<AttributeTag>,
    attrs: HashMap<AttributeTag, f32>,
}

impl Attributes {
    pub fn get(&self, tag: impl Into<AttributeTag>) -> f32 {
        self.attrs.get(&tag.into()).cloned().unwrap_or(0.0)
    }

    pub fn get_mut(&mut self, tag: impl Into<AttributeTag>) -> &mut f32 {
        let tag = tag.into();
        self.dirty.insert(tag.clone());
        self.attrs.entry(tag).or_insert(0.0)
    }

    pub fn set(&mut self, tag: impl Into<AttributeTag>, amount: f32) -> &mut Self {
        let tag = tag.into();
        *self.get_mut(tag) = amount;
        self
    }

    pub fn set_base(&mut self, tag: impl Into<AttributeTag>, amount: f32) -> &mut Self {
        let tag = tag.into();
        *self.get_mut(tag.clone()) = amount;
        let base_tag = AttributeTag::Modifier {
            modifier: Modifier::Base,
            target: Box::new(tag),
        };
        //info!("{:?}: {:?}", base_tag, amount);
        *self.get_mut(base_tag) = amount;
        self
    }

    pub fn add_stats(&mut self, changes: impl Iterator<Item = (impl Into<AttributeTag>, f32)>) {
        for (stat, change) in changes {
            let current = self.get_mut(stat);
            *current += change;
        }
    }

    pub fn remove_stats(&mut self, changes: impl Iterator<Item = (impl Into<AttributeTag>, f32)>) {
        for (stat, change) in changes {
            let current = self.get_mut(stat);
            *current -= change;
        }
    }
}

impl Default for Attributes {
    fn default() -> Self {
        let mut map: HashMap<AttributeTag, f32> = HashMap::new();
        for stat in Stat::iter() {
            use Stat::*;
            match stat {
                Health | CharacterResource | Gold => continue, // these stats dont need modifier stack, add xp and level?
                _ => (),
            }
            map.insert(
                AttributeTag::Modifier {
                    modifier: Modifier::Base,
                    target: Box::new(stat.clone().into()),
                },
                stat.get_base(),
            );
        }

        Self {
            dirty: default(),
            attrs: map,
        }
    }
}

#[derive(Event, Clone)]
pub struct HealthMitigatedEvent {
    pub change: i32,
    pub mitigated: u32,
    pub ability: Ability,
    pub attacker: Entity,
    pub defender: Entity,
    pub sensor: Entity,
    pub damage_type: DamageType,
    pub when: Instant,
}
