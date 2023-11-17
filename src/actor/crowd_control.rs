use std::{collections::BTreeMap, time::Duration};

use bevy::prelude::*;

use crate::{area::CCEvent, assets::Icons, game_manager::InGameSet};

#[derive(Debug, Clone, Reflect, Copy)]
pub struct CCInfo {
    pub cctype: CCType,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, Hash, PartialOrd, Ord)]
pub enum CCType {
    Stun,
    Root,
    Fear,
    Disarm,
    Silence,
    Cripple,
    //Slow, change to buff since affects a stat, proper CC's are for absolutes
}

impl CCType {
    pub fn get_icon(self, icons: &Res<Icons>) -> Handle<Image> {
        use CCType::*;
        match self {
            Stun => icons.frostbolt.clone().into(),
            Silence => icons.fireball.clone().into(),
            Root => icons.dash.clone().into(),
            _ => icons.basic_attack.clone().into(),
        }
    }
    pub fn to_text(&self) -> String {
        let str = match self {
            CCType::Stun => "STUNNED",
            CCType::Root => "ROOTED",
            CCType::Fear => "FEARED",
            CCType::Disarm => "DISARMED",
            CCType::Silence => "SILENCED",
            CCType::Cripple => "CRIPPLED",
        };
        str.to_string()
    }
}

impl std::fmt::Display for CCType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct CCPlugin;
impl Plugin for CCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (tick_ccs, apply_ccs).chain().in_set(InGameSet::Pre),
        );
    }
}

pub fn apply_ccs(mut targets_query: Query<&mut CCMap>, mut cc_events: EventReader<CCEvent>) {
    for event in cc_events.read() {
        let Ok(mut ccs) = targets_query.get_mut(event.target_entity) else {
            continue;
        };
        ccs.map.insert(
            event.ccinfo.cctype,
            Timer::new(
                Duration::from_millis((event.ccinfo.duration * 1000.0) as u64),
                TimerMode::Once,
            ),
        );
        sort_ccs(&mut ccs);
    }
}

pub fn tick_ccs(time: Res<Time>, mut query: Query<&mut CCMap>) {
    for mut ccs in &mut query {
        ccs.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}

pub fn sort_ccs(cc_map: &mut CCMap) {
    let mut sorted = Vec::from_iter(cc_map.map.clone());
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let new_map = sorted.into_iter().collect::<BTreeMap<CCType, Timer>>();
    cc_map.map = new_map;
}

#[derive(Component, Default, Debug, Clone)]
pub struct CCMap {
    pub map: BTreeMap<CCType, Timer>,
}
