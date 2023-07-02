use std::{time::Duration, collections::{BTreeMap}};

use bevy::prelude::*;

use crate::{ability::CCEvent, GameState, assets::Icons};


#[derive(Debug, Clone, Reflect, FromReflect, Copy)]
pub struct CCInfo{
    pub cctype: CCType,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, FromReflect, Hash, PartialOrd, Ord)]
pub enum CCType{
    Stun,
    Root,
    Fear,
    Disarm,
    Silence,
    Cripple,
    //Slow, change to buff since affects a stat, proper CC's are for absolutes
}

impl CCType{
    pub fn get_icon(self, icons: &Res<Icons>) -> Handle<Image>{
        use CCType::*;
        match self {
            Stun => icons.frostbolt.clone().into(),
            Cripple => icons.fireball.clone().into(),
            Root => icons.dash.clone().into(),
            _ => icons.basic_attack.clone().into(),
        }
    }
}

impl std::fmt::Display for CCType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct CCPlugin;
impl Plugin for CCPlugin{
    fn build(&self, app: &mut App) {

        app.add_systems((
            tick_ccs.run_if(in_state(GameState::InGame)),
            apply_ccs.run_if(in_state(GameState::InGame)),
        ).chain().in_base_set(CoreSet::PreUpdate));
    }
}

pub fn apply_ccs(
    mut targets_query: Query<&mut CCMap>,
    mut cc_events: EventReader<CCEvent>,
){
    for event in cc_events.iter(){
        if let Ok(mut ccs) = targets_query.get_mut(event.target_entity){
            ccs.map.insert(
                event.ccinfo.cctype,
                Timer::new(Duration::from_millis((event.ccinfo.duration * 1000.0) as u64), TimerMode::Once), 
            );
            sort_ccs(&mut ccs);
        }        
    }
}

pub fn tick_ccs(
    time: Res<Time>,
    mut query: Query<&mut CCMap>,
) {
    for mut ccs in &mut query {
        ccs.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
        sort_ccs(&mut ccs);
    }
}

pub fn sort_ccs(cc_map: &mut CCMap){
    let mut sorted = Vec::from_iter(cc_map.map.clone());
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let new_map = sorted.into_iter().collect::<BTreeMap<CCType, Timer>>();
    cc_map.map = new_map;
}

#[derive(Component, Default, Debug, Clone)]
pub struct CCMap {
    pub map: BTreeMap<CCType, Timer>,
}