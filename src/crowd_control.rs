use std::{collections::BTreeMap, time::Duration};

use bevy::prelude::*;

use crate::{actor::cast::Casting, area::queue::CCEvent, assets::Icons, session::director::InGameSet};

pub struct CCPlugin;
impl Plugin for CCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (tick_ccs, apply_ccs).chain().in_set(InGameSet::Pre),
        );
    }
}

#[derive(Debug, Clone, Reflect, Copy)]
pub struct CCInfo {
    pub cckind: CCKind,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, Hash, PartialOrd, Ord)]
pub enum CCKind {
    Stun,
    Root,
    Fear,
    Disarm,
    Silence,
    Cripple,
    //Slow, change to buff since affects a stat, proper CC's are for absolutes
    // UPDATE: probably want this to be an exception, make it a hybrid because it is useful to see displayed
}

impl CCKind {
    pub fn get_icon(self, icons: &Res<Icons>) -> Handle<Image> {
        use CCKind::*;
        match self {
            Stun => icons.frostbolt.clone().into(),
            Silence => icons.fireball.clone().into(),
            Root => icons.dash.clone().into(),
            _ => icons.basic_attack.clone().into(),
        }
    }
    pub fn to_text(&self) -> String {
        let str = match self {
            CCKind::Stun => "STUNNED",
            CCKind::Root => "ROOTED",
            CCKind::Fear => "FEARED",
            CCKind::Disarm => "DISARMED",
            CCKind::Silence => "SILENCED",
            CCKind::Cripple => "CRIPPLED",
        };
        str.to_string()
    }
    pub fn cancels_casts(&self) -> bool {
        self == &CCKind::Stun || self == &CCKind::Silence || self == &CCKind::Fear
    }
}

impl std::fmt::Display for CCKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn apply_ccs(mut targets_query: Query<(&mut CCMap, Option<&mut Casting>)>, mut cc_events: EventReader<CCEvent>) {
    for event in cc_events.read() {
        let Ok((mut ccs, casting_opt)) = targets_query.get_mut(event.target_entity) else { continue };
        // stop casting if we get CC'd
        if let Some(mut casting) = casting_opt {
            if event.ccinfo.cckind.cancels_casts() {
                *casting = Casting::default();
            }
        }
        ccs.map.insert(
            event.ccinfo.cckind,
            Timer::new(
                Duration::from_millis((event.ccinfo.duration * 1000.0) as u64),
                TimerMode::Once,
            ),
        );
        sort_ccs(&mut ccs);
    }
}

fn tick_ccs(time: Res<Time>, mut query: Query<&mut CCMap>) {
    for mut ccs in &mut query {
        ccs.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}

fn sort_ccs(cc_map: &mut CCMap) {
    let mut sorted = Vec::from_iter(cc_map.map.clone());
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let new_map = sorted.into_iter().collect::<BTreeMap<CCKind, Timer>>();
    cc_map.map = new_map;
}

#[derive(Component, Default, Debug, Clone)]
pub struct CCMap {
    pub map: BTreeMap<CCKind, Timer>,
}

impl CCMap {
    pub fn is_rooted(&self) -> bool {
        self.map.contains_key(&CCKind::Root)
    }
    pub fn is_stunned(&self) -> bool {
        self.map.contains_key(&CCKind::Stun)
    }
}
