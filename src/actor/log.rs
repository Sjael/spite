use bevy::ecs::{entity::Entity, event::Event};

use crate::{
    ability::{Ability, DamageType},
    stats::HealthMitigatedEvent,
};

pub struct DamageSum {
    total_change: i32,
    total_mitigated: u32,
    hit_amount: u32,
    sub_list: Vec<HealthMitigatedEvent>,
}

impl DamageSum {
    pub fn add_damage(&mut self, instance: HealthMitigatedEvent) {
        self.total_change += instance.change;
        self.total_mitigated += instance.mitigated;
        self.hit_amount += 1;
        self.sub_list.push(instance);
    }

    pub fn from_instance(instance: HealthMitigatedEvent) -> Self {
        DamageSum {
            total_change: instance.change,
            total_mitigated: instance.mitigated,
            hit_amount: 1,
            sub_list: vec![instance.clone()],
        }
    }

    pub fn total_change(&self) -> i32 {
        self.total_change
    }
    pub fn total_mitigated(&self) -> u32 {
        self.total_mitigated
    }
    pub fn hit_amount(&self) -> u32 {
        self.hit_amount
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogSide {
    Incoming,
    Outgoing,
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogType {
    Add,
    Stack,
}

// Change attacker to caster?
#[derive(Event, Debug)]
pub struct LogHit {
    pub sensor: Entity,
    pub attacker: Entity,
    pub defender: Entity,
    pub damage_type: DamageType,
    pub ability: Ability,
    pub change: i32,
    pub mitigated: u32,
    pub log_type: LogType,
    pub log_direction: LogSide,
}

impl LogHit {
    pub fn new(event: HealthMitigatedEvent, log_type: LogType, log_direction: LogSide) -> Self {
        LogHit {
            sensor: event.sensor,
            attacker: event.attacker,
            defender: event.defender,
            damage_type: event.damage_type,
            ability: event.ability,
            change: event.change,
            mitigated: event.mitigated,
            log_type,
            log_direction,
        }
    }
}
