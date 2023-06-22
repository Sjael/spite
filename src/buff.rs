use std::time::Duration;

use crate::{stats::{Stat}, ability::{BuffEvent, Ability}, player::BuffMap, game_manager::Team, GameState};
use bevy::prelude::*;


#[derive(Default, Clone, Debug, Reflect, FromReflect, Eq, PartialEq,)]
pub enum BuffType{
    #[default]
    Buff,
    Debuff,
}

#[derive(Default, Clone, Debug, Reflect, FromReflect, Eq, PartialEq,)]
pub enum BuffTargets{
    #[default]
    Allies,
    Enemies,
    All,
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub enum StackFalloff{
    Individual, // buff stacks drop one at a time,
    All, // buff stacks drop at the same time,
    Multiple(u32) // varying amount of falloff, pretty niche
}

#[derive( Clone, Debug, Reflect, FromReflect)]
pub enum StackRefresh{
    None, // adding a stack doesnt refesh any,
    All, // adding a stack refreshes all
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub struct BuffInfo{
    pub stat:Stat,
    pub amount: u32,
    pub max_stacks: u32,
    pub duration: f32,
    pub falloff: StackFalloff,
    pub refresh: StackRefresh,
    pub bufftargets: BuffTargets,
    pub bufftype: BuffType
}

impl Default for BuffInfo{
    fn default() -> Self {
        Self{
            stat: Stat::default(),
            amount: 1,
            max_stacks: 1,
            duration: 5.0,
            falloff: StackFalloff::All,
            refresh: StackRefresh::All,
            bufftargets: BuffTargets::Enemies,
            bufftype: BuffType::Debuff,
        }
    }
}

#[derive(Reflect, FromReflect, Debug, Clone)]
pub struct BuffInfoApplied{
    pub info: BuffInfo,
    pub stacks: u32,
    pub timer: Timer, // change to vec of timers for individual falloff
}


pub struct BuffPlugin;
impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            apply_buffs,
        ).in_set(OnUpdate(GameState::InGame)));
    }
}


pub fn apply_buffs(
    mut commands: Commands,
    mut targets_query: Query<(Entity, &mut BuffMap, &Team)>,
    mut buff_events: EventReader<BuffEvent>,
){
    for event in buff_events.iter(){
        if let Ok((entity, mut buffs, team)) = targets_query.get_mut(event.target){
            let originator = format!("{}v{}", event.buff_originator.index(), event.buff_originator.generation());
            let caster = format!("{}v{}", event.caster.index(), event.caster.generation());
            let event_buff_id = format!("{}_{}_{}", caster, originator, event.info.stat.to_string());
            dbg!(event_buff_id.clone());
            for (id, buff) in buffs.map.iter_mut(){
                if id == &event_buff_id{
                    buff.stacks += 1;
                    buff.timer.reset();
                }
            }
            if buffs.map.contains_key(&event_buff_id){
                println!("exists already");
            } else{
                buffs.map.insert(
                    event_buff_id,
                    BuffInfoApplied{
                        info: event.info.clone(),
                        stacks: 1,
                        timer: Timer::new(Duration::from_millis((event.info.duration * 1000.0) as u64), TimerMode::Once),
                    }                     
                );
            }
        }        
    }
}

//
// PLACE WITH STAT MODULE
//