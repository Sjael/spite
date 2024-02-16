use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::Rng;

use crate::{
    ability::{
        ticks::{PausesWhenEmpty, TickBehavior, TickKind},
        Ability, DamageType, MaxTargetsHit, TagInfo, Tags, TargetFilter, TargetSelection, TargetsHittable,
        TargetsInArea, UniqueTargetsHit,
    },
    actor::cast::{AbilityExtras, AbilityFireEvent, Caster},
    area::{AreaTimeline, CastStage},
    buff::{BuffInfo, BuffTargets},
    crowd_control::CCInfo,
    prelude::ActorState,
    session::team::Team,
    stats::{Attributes, Stat},
};

pub fn despawn_after_max_hits(mut commands: Commands, max_hits_query: Query<(Entity, &MaxTargetsHit)>) {
    for (entity, max_targets_hit) in max_hits_query.iter() {
        if max_targets_hit.current >= max_targets_hit.max {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// !!! Somehow I think this isn't working perfectly, some abilities are detecting
// Might just be the bevy_xbpd collision bug tho
pub fn area_apply_tags(
    mut sensor_query: Query<(
        Entity,
        &Team,
        &TargetsHittable,
        &Tags,
        Option<&mut MaxTargetsHit>,
        Option<&mut TickBehavior>,
        Option<&mut UniqueTargetsHit>,
        Option<&Caster>,
        Option<&DamageType>,
        Option<&Ability>,
    )>,
    mut targets_query: Query<(&Team, &ActorState)>,
    mut casters: Query<&mut Attributes>,
    mut health_events: EventWriter<HealthChangeEvent>,
    mut buff_events: EventWriter<BuffEvent>,
    mut cc_events: EventWriter<CCEvent>,
    mut cast_homing_events: EventWriter<AbilityFireEvent>,
) {
    'area: for (
        sensor_entity,
        sensor_team,
        targets_hittable,
        tags,
        mut max_targets_hit,
        tick_behavior,
        mut unique_targets_hit,
        caster,
        damage_type,
        ability,
    ) in &mut sensor_query
    {
        let mut hit_targets: Vec<Entity> = Vec::new();
        let ability = ability.unwrap_or(&Ability::BasicAttack);
        let damage_type = damage_type.unwrap_or(&DamageType::True);

        for target_entity in targets_hittable.list.iter() {
            let Ok((target_team, target_state)) = targets_query.get_mut(*target_entity) else { continue };
            if target_state.is_dead() {
                continue
            }
            if let Some(ref unique_targets_hit) = unique_targets_hit {
                if unique_targets_hit.already_hit.contains(target_entity) {
                    continue
                }
            }

            let caster = if let Some(caster) = caster {
                caster.0
            } else {
                sensor_entity // change to something random and funny like fg or 'God'
            };

            let on_same_team = sensor_team.0 == target_team.0;
            let mut hit_the_target = false;

            for taginfo in tags.iter() {
                match (taginfo, on_same_team) {
                    (TagInfo::Heal(amount), true) => {
                        hit_the_target = true;
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
                    (TagInfo::Damage(amount), false) => {
                        hit_the_target = true;
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
                    (TagInfo::Buff(buffinfo), on_same_team) => {
                        let buffing_ally = (buffinfo.bufftargets == BuffTargets::Allies) && on_same_team;
                        let debuffing_enemy = (buffinfo.bufftargets == BuffTargets::Enemies) && !on_same_team;
                        let buffing_anyone = buffinfo.bufftargets == BuffTargets::All;
                        let buff_to_send = BuffEvent {
                            info: buffinfo.clone(),
                            ability: *ability,
                            buff_originator: sensor_entity, // TODO can be a specific ability, or an item
                            caster,
                            target: *target_entity,
                        };
                        if buffing_ally || debuffing_enemy || buffing_anyone {
                            hit_the_target = true;
                            buff_events.send(buff_to_send);
                        }
                    }
                    // change to be able to cc allies later
                    (TagInfo::CC(ccinfo), false) => {
                        hit_the_target = true;
                        cc_events.send(CCEvent {
                            target_entity: *target_entity,
                            ccinfo: ccinfo.clone(),
                        });
                    }
                    (TagInfo::Homing(homing_ability), _) => {
                        hit_the_target = true;
                        cast_homing_events.send(AbilityFireEvent {
                            caster,
                            ability: homing_ability.clone(),
                            extras: vec![AbilityExtras::Homing(*target_entity)],
                        });
                    }
                    (TagInfo::ResourcePerTarget(change), false) => {
                        if let Ok(mut attrs) = casters.get_mut(caster) {
                            hit_the_target = true;
                            let max = attrs.get(Stat::CharacterResourceMax);
                            let stat = attrs.get_mut(Stat::CharacterResource);
                            *stat = (*stat + *change as f32).clamp(0.0, max);
                        }
                    }
                    (_, _) => (),
                }
            }

            if hit_the_target {
                hit_targets.push(*target_entity);
                if let Some(ref mut max_hits) = max_targets_hit {
                    max_hits.current += 1;
                    // if reached max hits, go to next area as this one will be despawned
                    if max_hits.current >= max_hits.max {
                        continue 'area
                    }
                }
            }
        }

        // if area has ticks, log in individual timers
        if let Some(mut tick_behavior) = tick_behavior {
            let interval = tick_behavior.interval.clone();
            if let TickKind::Individual(ref mut individual_timers) = tick_behavior.kind {
                for hit_entity in hit_targets.iter() {
                    individual_timers.insert(
                        *hit_entity,
                        Timer::new(Duration::from_secs_f32(interval), TimerMode::Once),
                    );
                }
            }
        }
        // if same target cant be hit twice ever (ra 1), log them
        if let Some(ref mut unique_targets_hit) = unique_targets_hit {
            for hit_entity in hit_targets.iter() {
                unique_targets_hit.already_hit.push(*hit_entity);
            }
        }
    }
}

pub fn area_queue_targets(
    mut sensor_query: Query<(
        &TargetsInArea,
        &mut TargetsHittable,
        Option<&TickBehavior>,
        Option<&TargetFilter>,
        Option<&AreaTimeline>,
    )>,
) {
    for (targets_in_area, mut targets_hittable, tick_behavior, filter, timeline) in sensor_query.iter_mut() {
        targets_hittable.list = Vec::new();
        if let Some(timeline) = timeline {
            if timeline.stage != CastStage::Firing {
                continue
            }
        }
        if targets_in_area.list.is_empty() {
            continue
        }
        if let Some(tick_behavior) = tick_behavior {
            match tick_behavior.kind {
                TickKind::Static(ref static_timer) => {
                    if !static_timer.finished() {
                        continue
                    }
                    if let Some(filter) = filter {
                        targets_hittable.list = filter.list.clone();
                    } else {
                        targets_hittable.list = targets_in_area.list.clone();
                    }
                }
                TickKind::Individual(ref individual_timers) => {
                    for target_entity in targets_in_area.list.iter() {
                        if let Some(filter) = filter {
                            if !filter.list.contains(target_entity) {
                                continue
                            }
                        }
                        let hasnt_been_hit_or_interval_over = match individual_timers.get(&target_entity) {
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
    mut sensor_query: Query<(&mut TargetFilter, &GlobalTransform, &TargetsInArea, Entity)>,
    changed_sensors: Query<Entity, Changed<TargetsInArea>>,
    target_query: Query<&GlobalTransform>, // add threat stat later
) {
    for (mut filter, sensor_transform, targets_in_area, sensor_entity) in sensor_query.iter_mut() {
        if targets_in_area.list.is_empty() {
            filter.list = Vec::new();
            continue
        }
        let mut targets_thru_filter: Vec<Entity> = Vec::new();
        match filter.target_selection {
            TargetSelection::Closest => {
                let num_of_targets = filter.number_of_targets;

                let mut closest_targets: Vec<(f32, Entity)> = Vec::new();
                for target_entity in targets_in_area.list.iter() {
                    let Ok(target_transform) = target_query.get(*target_entity) else { continue };
                    let relative_translation = target_transform.translation() - sensor_transform.translation();
                    closest_targets.push((relative_translation.length(), *target_entity));
                }
                closest_targets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less));

                let (_, closest_entities): (Vec<_>, Vec<_>) =
                    closest_targets.into_iter().take(num_of_targets as usize).unzip();
                targets_thru_filter = closest_entities;
            }
            TargetSelection::Random => {
                // only make random selection when the targets change, instead of every frame
                let Ok(_) = changed_sensors.get(sensor_entity) else { continue };
                // can hit the same target twice lol, should remove from array and gen again but idc
                let mut rng = rand::thread_rng();
                for _ in 0..filter.number_of_targets {
                    let random_target_index = rng.gen_range(0..targets_in_area.list.len());
                    let from_list = targets_in_area.list.get(random_target_index).unwrap();
                    targets_thru_filter.push(*from_list);
                }
            }
            _ => (),
        }
        filter.list = targets_thru_filter
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
            continue
        };

        let ((area_entity, mut targets_in_area), target_entity) = if let Ok(target) = targets_query.get(*potential) {
            (sensor, target)
        } else {
            continue
        };
        targets_in_area.list.push(target_entity);
        area_events.send(AreaOverlapEvent {
            sensor: area_entity,
            target: target_entity,
            overlap: AreaOverlapType::Entered,
        });
    }
    for CollisionEnded(entity1, entity2) in collision_ends.read() {
        let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(*entity1) {
            (sensor, entity2)
        } else if let Ok(sensor) = sensor_query.get_mut(*entity2) {
            (sensor, entity1)
        } else {
            continue
        };

        let ((area_entity, mut targets_in_area), target_entity) = if let Ok(target) = targets_query.get(*potential) {
            (sensor, target)
        } else {
            continue
        };
        if let Some(index) = targets_in_area.list.iter().position(|x| *x == target_entity) {
            targets_in_area.list.remove(index);
            area_events.send(AreaOverlapEvent {
                sensor: area_entity,
                target: target_entity,
                overlap: AreaOverlapType::Exited,
            });
        }
    }
}

pub fn tick_hit_timers(
    time: Res<Time>,
    mut area_timers: Query<(&TargetsInArea, &mut TickBehavior, Option<&PausesWhenEmpty>)>,
) {
    for (targets_in_area, mut tick_behavior, pauses) in &mut area_timers {
        match tick_behavior.kind {
            TickKind::Individual(ref mut individual_timers) => {
                // tick per-target timer and retain it if not finished
                individual_timers.retain(|_entity, hittimer| {
                    hittimer.tick(time.delta());
                    !hittimer.finished()
                });
            }
            TickKind::Static(ref mut static_timer) => {
                // tick whole ability timer unless empty and pauses (towers)
                if pauses.is_some() && targets_in_area.list.is_empty() {
                    static_timer.set_mode(TimerMode::Once);
                } else {
                    static_timer.set_mode(TimerMode::Repeating);
                }
                static_timer.tick(time.delta());
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
