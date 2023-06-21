use std::time::Duration;

use crate::{stats::{Stat}, ability::{BuffEvent, Ability}, player::BuffMap, game_manager::Team};
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

#[derive(Clone, Debug, Reflect, FromReflect)]
pub struct AppliedBuff{
    caster: Entity,
    ability: Ability,
    info: BuffInfo,
}

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
    mut targets_query: Query<(Entity, &mut BuffMap, &Team)>,
    mut buff_events: EventReader<BuffEvent>,
){
    for event in buff_events.iter(){
        if let Ok((entity, mut buffs, team)) = targets_query.get_mut(event.target){
            let buff_id = event.info.stat.to_string();
            if buffs.map.contains_key(&buff_id){
                if event.team == *team{
                    
                }
            } else{
                buffs.map.insert(
                    buff_id,
                    Timer::new(Duration::from_millis((event.info.duration * 1000.0) as u64), TimerMode::Once), 
                );
            }
        }        
    }
}

//
// PLACE WITH STAT MODULE
//