use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{component::Component, entity::Entity, reflect::ReflectComponent},
    reflect::Reflect,
    time::{Timer, TimerMode},
};

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct TickBehavior {
    pub ticks: Ticks,
    pub kind: TickKind,
    pub interval: f32,
}

impl TickBehavior {
    pub fn set_timer(&mut self) {
        match self.kind {
            TickKind::Static(ref mut timer) => {
                timer.set_duration(Duration::from_millis(self.interval as u64));
            }
            _ => (),
        }
    }

    pub fn new_static_from_ticks(ticks: Ticks, lifetime: f32) -> Self {
        let interval = match ticks {
            Ticks::Multiple(tick_num) => lifetime / tick_num as f32,
            Ticks::Once => lifetime,
            _ => 1.0,
        };
        Self {
            ticks,
            kind: TickKind::static_timer(interval),
            interval,
        }
    }
    pub fn new_individual_from_ticks(ticks: Ticks, lifetime: f32) -> Self {
        let interval = match ticks {
            Ticks::Multiple(tick_num) => lifetime / tick_num as f32,
            Ticks::Once => lifetime,
            _ => 1.0,
        };
        Self {
            ticks,
            kind: TickKind::individual(),
            interval,
        }
    }

    pub fn new_static(interval: f32) -> Self {
        Self {
            interval,
            kind: TickKind::static_timer(interval),
            ticks: Ticks::Unlimited,
        }
    }
    pub fn new_individual(interval: f32) -> Self {
        Self {
            interval,
            kind: TickKind::individual(),
            ticks: Ticks::Unlimited,
        }
    }
}

impl Default for TickBehavior {
    fn default() -> Self {
        Self::new_static(1.0)
    }
}

#[derive(Reflect, Debug, Clone)]
pub enum TickKind {
    Static(Timer),
    Individual(HashMap<Entity, Timer>),
}

impl TickKind {
    pub fn static_timer(interval: f32) -> Self {
        Self::Static(Timer::new(
            Duration::from_millis(interval as u64 * 1000),
            TimerMode::Repeating,
        ))
    }
    pub fn individual() -> Self {
        Self::Individual(HashMap::default())
    }
}

impl Default for TickKind {
    fn default() -> Self {
        Self::Static(Timer::default())
    }
}

#[derive(Debug, Clone, Default, Reflect)]
pub enum Ticks {
    #[default]
    Once,
    Multiple(u32),
    Unlimited,
}

#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct PausesWhenEmpty;
