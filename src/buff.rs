use std::{collections::HashMap, time::Duration};

use crate::{
    ability::{Ability, BuffEvent},
    game_manager::Team,
    stats::{AttributeTag, Attributes, Health, MovementSpeed, Stat},
    GameState,
};
use bevy::prelude::*;

#[derive(Default, Clone, Copy, Debug, Reflect, FromReflect, Eq, PartialEq)]
pub enum BuffType {
    #[default]
    Buff,
    Debuff,
}

#[derive(Default, Clone, Debug, Reflect, FromReflect, Eq, PartialEq)]
pub enum BuffTargets {
    #[default]
    Allies,
    Enemies,
    All,
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub enum StackFalloff {
    Individual,    // buff stacks drop one at a time,
    All,           // buff stacks drop at the same time,
    Multiple(u32), // varying amount of falloff, pretty niche
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub enum StackRefresh {
    None, // adding a stack doesnt refesh any,
    All,  // adding a stack refreshes all
}

#[derive(Clone, Debug)]
pub struct BuffInfo {
    pub stat: AttributeTag,
    pub amount: u32,
    pub max_stacks: u32,
    pub duration: f32,
    pub falloff: StackFalloff,
    pub refresh: StackRefresh,
    pub bufftargets: BuffTargets,
    pub bufftype: BuffType,
}

impl Default for BuffInfo {
    fn default() -> Self {
        Self {
            stat: AttributeTag::Stat(Stat::Health),
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

#[derive(Debug, Clone)]
pub struct BuffInfoApplied {
    pub info: BuffInfo,
    pub stacks: u32,
    pub timer: Timer, // change to vec of timers for individual falloff
}

pub struct BuffPlugin;
impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuffAddEvent>();
        app.add_event::<BuffStackEvent>();

        /*
        app.add_systems(apply_buffs)
            .in_set(OnUpdate(GameState::InGame));
        */
    }
}

pub struct BuffStackEvent {
    pub id: String,
    pub stacks: u32,
    pub target: Entity,
}

pub struct BuffAddEvent {
    pub id: String,
    pub target: Entity,
    pub bufftype: BuffType,
    pub duration: f32,
}

pub fn apply_buffs(
    mut commands: Commands,
    mut targets_query: Query<(Entity, &mut BuffMap, &Team, &mut Attributes)>,
    mut buff_events: EventReader<BuffEvent>,
    mut stack_events: EventWriter<BuffStackEvent>,
    mut add_events: EventWriter<BuffAddEvent>,
) {
    for event in buff_events.iter() {
        if let Ok((entity, mut buffs, team, mut attribute)) = targets_query.get_mut(event.target) {
            let originator = format!(
                "{}v{}",
                event.buff_originator.index(),
                event.buff_originator.generation()
            );
            let caster = format!("{}v{}", event.caster.index(), event.caster.generation());
            //let buff_id = format!("{}_{}_{}", caster, originator, event.info.stat.to_string());

            /*
            if buffs.map.contains_key(&buff_id) {
                let Some(applied) = buffs.map.get_mut(&buff_id) else {continue};
                if applied.info.max_stacks > applied.stacks {
                    applied.stacks += 1;
                }
                applied.timer.reset(); // reset timer, TODO need individual stack timer support
                stack_events.send(BuffStackEvent {
                    id: buff_id,
                    stacks: applied.stacks,
                    target: event.target,
                });
            } else {
                buffs.map.insert(
                    buff_id.clone(),
                    BuffInfoApplied {
                        info: event.info.clone(),
                        stacks: 1,
                        timer: Timer::new(
                            Duration::from_millis((event.info.duration * 1000.0) as u64),
                            TimerMode::Once,
                        ),
                    },
                );
                add_events.send(BuffAddEvent {
                    id: buff_id,
                    target: event.target,
                    bufftype: event.info.bufftype,
                    duration: event.info.duration,
                });
            }

            attribute.get_mut(event.info.stat) += event.info.amount;
            */
        }
    }
}

pub fn tick_buffs(time: Res<Time>, mut query: Query<&mut BuffMap>) {
    for mut buffs in &mut query {
        buffs.map.retain(|_, buff| {
            buff.timer.tick(time.delta());
            !buff.timer.finished()
        });
    }
}

//
// PLACE WITH STAT MODULE
//

#[derive(Component, Default, Debug, Clone)]
pub struct BuffMap {
    pub map: HashMap<AttributeTag, BuffInfoApplied>, // Create buff id from entity-ability/item-positive, orc2-spear-debuff aka who it comes from
}
