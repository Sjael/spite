use std::{time::Duration, collections::HashMap};

use bevy::prelude::*;

use crate::{ability::CCEvent};


#[derive(Debug, Clone, Reflect, FromReflect, Copy)]
pub struct CCInfo{
    pub cctype: CCType,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, FromReflect, Hash)]
pub enum CCType{
    Stun,
    Root,
    Fear,
    Disarm,
    Silence,
    //Slow, change to buff since affects a stat, proper CC's are for absolutes
    Cripple,
}

impl std::fmt::Display for CCType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn apply_ccs(
    mut commands: Commands,
    mut targets_query: Query<(Entity, &mut CCMap)>,
    mut cc_events: EventReader<CCEvent>,
){
    for event in cc_events.iter(){
        if let Ok((entity, mut cc)) = targets_query.get_mut(event.target_entity){
            cc.map.insert(
                event.ccinfo.cctype,
                Timer::new(Duration::from_millis((event.ccinfo.duration * 1000.0) as u64), TimerMode::Once), 
            );
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
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CCMap {
    pub map: HashMap<CCType, Timer>,
}