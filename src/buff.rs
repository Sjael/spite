use std::time::Duration;

use crate::{stats::{Stat}, ability::BuffEvent, player::BuffMap};
use bevy::prelude::*;


#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct BuffInfoTest{
    pub stat:Stat,
    pub duration: f32,
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct BuffInfo{
    pub stat:Stat,
    pub max_stacks: u32,
    pub duration: u32,
    pub falloff: StackFalloff,
    pub refresh: StackRefresh,
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct UIBuffId(pub String);

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub enum StackFalloff{
    None, // buff stacks drop one at a time,
    #[default]
    All, // buff stacks drop at the same time,
    Multiple(u32) // varying amount of falloff, pretty niche
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub enum StackRefresh{
    None, // adding a stack doesnt refesh any,
    #[default]
    All, // adding a stack refreshes all
    Multiple(u32),
}

pub fn apply_buffs(
    mut commands: Commands,
    mut targets_query: Query<(Entity, &mut BuffMap)>,
    mut buff_events: EventReader<BuffEvent>,
){
    for event in buff_events.iter(){
        if let Ok((entity, mut buffs)) = targets_query.get_mut(event.entity){
            buffs.map.insert(
                event.info.stat.to_string(),
                Timer::new(Duration::from_millis((event.info.duration * 1000.0) as u64), TimerMode::Once), 
            );
        }        
    }
}

//
// PLACE WITH STAT MODULE
//