use bevy::prelude::*;

use bevy_xpbd_3d::prelude::*;
use rand::Rng;
use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use crate::{
    ability::{
        buff::{BuffInfo, BuffTargets},
        cast::{Caster, FireHomingEvent},
        crowd_control::CCInfo,
        timeline::{AreaTimeline, DeployAreaStage},
        Ability, DamageType, FilteredTargets, FiringInterval, MaxTargetsHit, PausesWhenEmpty,
        TagInfo, Tags, TargetFilter, TargetSelection, TargetsHittable, TargetsInArea, TickBehavior,
        Ticks, UniqueTargetsHit,
    },
    session::team::Team,
};

pub fn despawn_after_max_hits(
    mut commands: Commands,
    max_hits_query: Query<(Entity, &MaxTargetsHit)>,
) {
    for (entity, max_targets_hit) in max_hits_query.iter() {
        if max_targets_hit.current >= max_targets_hit.max {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn area_apply_tags(
    mut sensor_query: Query<(
        Entity,
        &Team,
        &TargetsHittable,
        &Tags,
        Option<&DamageType>,
        Option<&Caster>,
        Option<&Parent>,
        Option<&FiringInterval>,
        Option<&mut MaxTargetsHit>,
        Option<&mut TickBehavior>,
        Option<&mut UniqueTargetsHit>,
        Option<&Ability>,
    )>,
    mut targets_query: Query<(Entity, &Team)>,
    mut health_events: EventWriter<HealthChangeEvent>,
    mut buff_events: EventWriter<BuffEvent>,
    mut cc_events: EventWriter<CCEvent>,
    mut cast_homing_events: EventWriter<FireHomingEvent>,
) {
    for (
        sensor_entity,
        sensor_team,
        targets_hittable,
        tags,
        damage_type,
        caster,
        _parent,
        interval,
        mut max_targets_hit,
        mut tick_behavior,
        mut unique_targets_hit,
        ability,
    ) in &mut sensor_query
    {
        let mut hit_targets: Vec<Entity> = Vec::new();
        let ability = ability.unwrap_or(&Ability::BasicAttack);
        let damage_type = damage_type.unwrap_or(&DamageType::True);
        for target_entity in targets_hittable.list.iter() {
            let Ok((_, target_team)) = targets_query.get_mut(*target_entity) else {
                continue;
            };
            let caster = if let Some(caster) = caster {
                caster.0
            } else {
                sensor_entity // change to something random and funny like fg or 'God'
            };
            let on_same_team = sensor_team.0 == target_team.0;
            if let Some(ref unique_targets_hit) = unique_targets_hit {
                if unique_targets_hit.already_hit.contains(target_entity) {
                    continue;
                }
            }
            let mut hit_a_target = false;
            for taginfo in tags.list.iter() {
                match taginfo {
                    TagInfo::Heal(amount) => {
                        if on_same_team {
                            hit_a_target = true;
                            let health_change = HealthChangeEvent {
                                amount: *amount,
                                damage_type: damage_type.clone(),
                                ability: ability.clone(),
                                attacker: caster,
                                defender: *target_entity,
                                sensor: sensor_entity,
                                when: Instant::now(),
                            };
                            health_events.send(health_change);
                        }
                    }
                    TagInfo::Damage(amount) => {
                        if !on_same_team {
                            hit_a_target = true;
                            let health_change = HealthChangeEvent {
                                amount: -amount,
                                damage_type: damage_type.clone(),
                                ability: ability.clone(),
                                attacker: caster,
                                defender: *target_entity,
                                sensor: sensor_entity,
                                when: Instant::now(),
                            };
                            health_events.send(health_change);
                        }
                    }
                    TagInfo::Buff(ref buffinfo) => {
                        let buffing_ally =
                            (buffinfo.bufftargets == BuffTargets::Allies) && on_same_team;
                        let debuffing_enemy =
                            (buffinfo.bufftargets == BuffTargets::Enemies) && !on_same_team;
                        let buffing_anyone = buffinfo.bufftargets == BuffTargets::All;
                        let buff_to_send = BuffEvent {
                            info: buffinfo.clone(),
                            ability: *ability,
                            buff_originator: sensor_entity, // TODO can be a specific ability, or an item
                            caster: caster,
                            target: *target_entity,
                        };
                        if buffing_ally || debuffing_enemy || buffing_anyone {
                            hit_a_target = true;
                            buff_events.send(buff_to_send);
                        }
                    }
                    TagInfo::CC(ref ccinfo) => {
                        if !on_same_team {
                            hit_a_target = true;
                            cc_events.send(CCEvent {
                                target_entity: *target_entity,
                                ccinfo: ccinfo.clone(),
                            });
                        }
                    }
                    TagInfo::Homing(ref projectile_to_spawn) => {
                        hit_a_target = true;
                        cast_homing_events.send(FireHomingEvent {
                            caster: caster,
                            ability: projectile_to_spawn.clone(),
                            target: *target_entity,
                        });
                    }
                }
            }
            if hit_a_target {
                hit_targets.push(*target_entity);
                if let Some(ref mut max_hits) = max_targets_hit {
                    max_hits.current += 1;
                    if max_hits.current >= max_hits.max {
                        return;
                    }
                }
            }
        }
        if let Some(mut tick_behavior) = tick_behavior {
            if let Some(interval) = interval {
                if let TickBehavior::Individual(ref mut individual_timers) = *tick_behavior {
                    for got_hit in hit_targets.iter() {
                        individual_timers.map.insert(
                            *got_hit,
                            Timer::new(Duration::from_secs_f32(interval.0), TimerMode::Once),
                        );
                    }
                }
            }
        }
        for got_hit in hit_targets.iter() {
            if let Some(ref mut unique_targets_hit) = unique_targets_hit {
                unique_targets_hit.already_hit.push(*got_hit);
            }
        }
    }
}

pub fn area_queue_targets(
    mut sensor_query: Query<(
        &TargetsInArea,
        &mut TargetsHittable,
        Option<&TickBehavior>,
        Option<&FilteredTargets>,
        Option<&AreaTimeline>,
    )>,
) {
    for (targets_in_area, mut targets_hittable, tick_behavior, filtered_targets, timeline) in
        sensor_query.iter_mut()
    {
        targets_hittable.list = Vec::new();
        if let Some(timeline) = timeline {
            if timeline.stage != DeployAreaStage::Firing {
                continue;
            }
        }
        if targets_in_area.list.is_empty() {
            continue;
        }
        if let Some(tick_behavior) = tick_behavior {
            match *tick_behavior {
                TickBehavior::Static(ref static_timer) => {
                    if !static_timer.finished() {
                        continue;
                    }
                    if let Some(filtered_targets) = filtered_targets {
                        targets_hittable.list = filtered_targets.list.clone();
                    } else {
                        targets_hittable.list = targets_in_area.list.clone();
                    }
                }
                TickBehavior::Individual(ref individual_timers) => {
                    for target_entity in targets_in_area.list.iter() {
                        if let Some(filtered_targets) = filtered_targets {
                            if !filtered_targets.list.contains(target_entity) {
                                continue;
                            }
                        }
                        let hasnt_been_hit_or_interval_over =
                            match individual_timers.map.get(&target_entity) {
                                Some(timer) => timer.finished(),
                                None => true,
                            };
                        if hasnt_been_hit_or_interval_over {
                            targets_hittable.list.push(*target_entity);
                        }
                    }
                }
            }
        } else {
            targets_hittable.list = targets_in_area.list.clone(); // any that enter get hit
        }
    }
}

pub fn filter_targets(
    mut sensor_query: Query<(
        &TargetFilter,
        &GlobalTransform,
        &TargetsInArea,
        &mut FilteredTargets,
        Entity,
    )>,
    changed_sensors: Query<Entity, Changed<TargetsInArea>>,
    target_query: Query<&GlobalTransform>, // add threat stat later
) {
    for (target_filter, sensor_transform, targets_in_area, mut filtered_targets, sensor_entity) in
        sensor_query.iter_mut()
    {
        if targets_in_area.list.is_empty() {
            filtered_targets.list = Vec::new();
            continue;
        }
        let mut targets_thru_filter: Vec<Entity> = Vec::new();
        match target_filter.target_selection {
            TargetSelection::Closest => {
                let num_of_targets = target_filter.number_of_targets;

                let mut closest_targets: Vec<(f32, Entity)> = Vec::new();
                for target_entity in targets_in_area.list.iter() {
                    let Ok(target_transform) = target_query.get(*target_entity) else {
                        continue;
                    };
                    let relative_translation =
                        target_transform.translation() - sensor_transform.translation();
                    closest_targets.push((relative_translation.length(), *target_entity));
                }
                closest_targets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less));

                let (_, closest_entities): (Vec<_>, Vec<_>) = closest_targets
                    .into_iter()
                    .take(num_of_targets as usize)
                    .unzip();
                targets_thru_filter = closest_entities;
            }
            TargetSelection::Random => {
                // only make random selection when the targets change, instead of every frame
                let Ok(_) = changed_sensors.get(sensor_entity) else {
                    continue;
                };
                // can hit the same target twice lol, should remove from array and gen again but idc
                let mut rng = rand::thread_rng();
                for _ in 0..target_filter.number_of_targets {
                    let random_target_index = rng.gen_range(0..targets_in_area.list.len());
                    let from_list = targets_in_area.list.get(random_target_index).unwrap();
                    targets_thru_filter.push(*from_list);
                }
            }
            _ => (),
        }
        filtered_targets.list = targets_thru_filter
    }
}

pub fn catch_collisions(
    targets_query: Query<Entity, Without<Sensor>>,
    mut sensor_query: Query<(Entity, &mut TargetsInArea), With<Sensor>>,
    mut collision_starts: EventReader<CollisionStarted>,
    mut collision_ends: EventReader<CollisionEnded>,
    mut area_events: EventWriter<AreaOverlapEvent>,
) {
    for CollisionStarted(entity1, entity2) in collision_starts.read() {
        let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(*entity1) {
            (sensor, entity2)
        } else if let Ok(sensor) = sensor_query.get_mut(*entity2) {
            (sensor, entity1)
        } else {
            continue;
        };

        let ((area_entity, mut targets_in_area), target_entity) =
            if let Ok(target) = targets_query.get(*potential) {
                (sensor, target)
            } else {
                continue;
            };
        targets_in_area.list.push(target_entity);
        area_events.send(AreaOverlapEvent {
            sensor: area_entity,
            target: target_entity,
            overlap: AreaOverlapType::Entered,
        });
        // if !targets_in_area.list.is_empty() {
        //     dbg!(target_entity);
        //     dbg!(area_entity);
        // }
    }
    for CollisionEnded(entity1, entity2) in collision_ends.read() {
        let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(*entity1) {
            (sensor, entity2)
        } else if let Ok(sensor) = sensor_query.get_mut(*entity2) {
            (sensor, entity1)
        } else {
            continue;
        };

        let ((area_entity, mut targets_in_area), target_entity) =
            if let Ok(target) = targets_query.get(*potential) {
                (sensor, target)
            } else {
                continue;
            };
        if let Some(index) = targets_in_area
            .list
            .iter()
            .position(|x| *x == target_entity)
        {
            targets_in_area.list.remove(index);
            area_events.send(AreaOverlapEvent {
                sensor: area_entity,
                target: target_entity,
                overlap: AreaOverlapType::Exited,
            });
        }
    }
}

// fn get_sensors(entity1: Entity, entity2: Entity, sensor_query: &mut Query<(Entity, &mut TargetsInArea), With<Sensor>>, targets_query: &Query<Entity, Without<Sensor>>) -> Option<(Entity, Entity, &mut TargetsInArea)> {
//     let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(entity1) {
//         (sensor, entity2)
//     } else if let Ok(sensor) = sensor_query.get_mut(entity2) {
//         (sensor, entity1)
//     } else {
//         return None
//     };

//     let ((area_entity, mut targets_in_area), target_entity) = if let Ok(target) = targets_query.get(potential) {
//         (sensor, target)
//     } else {
//         return None
//     };
//     Some((area_entity, target_entity, &mut *targets_in_area))
// }

pub fn tick_hit_timers(
    time: Res<Time>,
    mut area_timers: Query<(
        &TargetsInArea,
        &Ticks,
        &FiringInterval,
        &mut TickBehavior,
        Option<&PausesWhenEmpty>,
    )>,
) {
    for (targets_in_area, ticks, _interval, mut tick_behavior, pauses) in &mut area_timers {
        match *tick_behavior {
            TickBehavior::Individual(ref mut individual_timers) => {
                // tick per-target timer and retain it if not finished
                individual_timers.map.retain(|_entity, hittimer| {
                    hittimer.tick(time.delta());
                    !hittimer.finished()
                });
            }
            TickBehavior::Static(ref mut static_timer) => {
                // tick whole ability timer unless empty and pauses (towers)
                if pauses.is_some() && targets_in_area.list.is_empty() {
                    static_timer.set_mode(TimerMode::Once);
                } else {
                    static_timer.set_mode(TimerMode::Repeating);
                }
                static_timer.tick(time.delta());
                match *ticks {
                    Ticks::Multiple(mut amount) => {
                        if static_timer.just_finished() {
                            amount = amount - 1;
                        }
                        if amount == 0 {
                            static_timer.pause();
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

#[derive(Event, Clone)]
pub struct HealthChangeEvent {
    pub amount: f32,
    pub damage_type: DamageType,
    pub ability: Ability,
    pub attacker: Entity,
    pub defender: Entity,
    pub sensor: Entity,
    pub when: Instant,
}

#[derive(Event)]
pub struct BuffEvent {
    pub info: BuffInfo,
    pub target: Entity,
    pub buff_originator: Entity,
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Event)]
pub struct CCEvent {
    pub target_entity: Entity,
    pub ccinfo: CCInfo,
}

#[derive(PartialEq, Eq)]
pub enum AreaOverlapType {
    Entered,
    Exited,
}

#[derive(Event)]
pub struct AreaOverlapEvent {
    pub sensor: Entity,
    pub target: Entity,
    pub overlap: AreaOverlapType,
}
