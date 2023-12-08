use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::Rng;
use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    ability::{
        bundles::Caster, Ability, AbilityTooltip, AreaLifetime, AreaTimeline, DamageType,
        DeployAreaStage, FilteredTargets, FiringInterval, MaxTargetsHit, PausesWhenEmpty, TagInfo,
        Tags, TargetFilter, TargetSelection, TargetsHittable, TargetsInArea, TickBehavior, Ticks,
        UniqueTargetsHit,
    },
    actor::{
        buff::{BuffInfo, BuffTargets},
        crowd_control::CCInfo,
        FireHomingEvent,
    },
    collision_masks::Team,
};
use homing::track_homing;

use self::non_damaging::*;

#[derive(Component)]
pub struct ProcMap(pub HashMap<Ability, Vec<AbilityBehavior>>);

pub enum AbilityBehavior {
    Homing,
    OnHit,
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

pub struct AreaPlugin;
impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AbilityTooltip>();

        app.add_event::<HealthChangeEvent>();
        app.add_event::<AreaOverlapEvent>();
        app.add_event::<BuffEvent>();
        app.add_event::<CCEvent>();

        app.add_systems(PreUpdate, (apply_interval, catch_collisions));
        app.add_systems(
            Update,
            (
                tick_lifetime,
                tick_hit_timers,
                track_homing,
                add_health_bar_detect_colliders,
                focus_objective_health,
                tick_timeline.before(area_queue_targets),
            ),
        );
        app.add_systems(
            Update,
            (
                filter_targets,
                area_queue_targets,
                area_apply_tags,
                despawn_after_max_hits,
            )
                .chain(),
        );
    }
}

fn despawn_after_max_hits(mut commands: Commands, max_hits_query: Query<(Entity, &MaxTargetsHit)>) {
    for (entity, max_targets_hit) in max_hits_query.iter() {
        if max_targets_hit.current >= max_targets_hit.max {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn area_apply_tags(
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
        parent,
        interval,
        mut max_targets_hit,
        mut tick_behavior,
        mut unique_targets_hit,
        ability,
    ) in &mut sensor_query
    {
        let mut targets_that_got_hit: Vec<Entity> = Vec::new();
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
                targets_that_got_hit.push(*target_entity);
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
                    for got_hit in targets_that_got_hit.iter() {
                        individual_timers.map.insert(
                            *got_hit,
                            Timer::new(Duration::from_millis(interval.0 as u64), TimerMode::Once),
                        );
                    }
                }
            }
        }
        for got_hit in targets_that_got_hit.iter() {
            if let Some(ref mut unique_targets_hit) = unique_targets_hit {
                unique_targets_hit.already_hit.push(*got_hit);
            }
        }
    }
}

fn area_queue_targets(
    mut sensor_query: Query<(
        &TargetsInArea,
        &mut TargetsHittable,
        Option<&TickBehavior>,
        Option<&FilteredTargets>,
        &AreaTimeline,
    )>,
) {
    for (targets_in_area, mut targets_hittable, tick_behavior, filtered_targets, timeline) in
        sensor_query.iter_mut()
    {
        targets_hittable.list = Vec::new();
        if timeline.stage != DeployAreaStage::Firing {
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

fn filter_targets(
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
                // can hit the same target twice lol, should remove from array and gen again but
                // idc
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
/*
ullr axe
    TargetsInArea
    Tags
    max hits
    TargetsHittable

Ra heal
    TargetsInArea
    Tags
    Ticks individually
    TargetsHittable

Anubis 3
    TargetsInArea
    Tags
    Ticks statically
    TargetsHittable

Tower
    TargetsInArea
    Tags
    Ticks statically
    PausesWhenEmpty (ready to fire on enter) (doesnt reset unless it fires)
    TargetsHittable
    FilterTargets::Closest (1)

 */

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

fn catch_collisions(
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

fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut AreaLifetime, Entity)>,
) {
    for (mut lifetime, entity) in lifetimes.iter_mut() {
        //dbg!(lifetime.clone());
        lifetime.seconds -= time.delta_seconds_f64();
        if lifetime.seconds <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn tick_timeline(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut AreaTimeline, Entity, &mut Visibility)>,
) {
    for (mut timeline, entity, mut vis) in lifetimes.iter_mut() {
        //dbg!(lifetime.clone());
        timeline.timer.tick(time.delta());
        if timeline.timer.finished() {
            use crate::ability::DeployAreaStage::*;
            let new_stage = timeline.stage.get_next_stage();
            let new_time = timeline.blueprint.get(&new_stage).unwrap_or(&0.0).clone();
            timeline.timer = Timer::new(Duration::from_secs_f32(new_time), TimerMode::Once);
            if timeline.stage == Recovery {
                commands.entity(entity).despawn_recursive();
            } else if timeline.stage == Windup {
                *vis = Visibility::Visible;
            }
        }
    }
}

fn apply_interval(
    mut area_timers: Query<(&FiringInterval, &mut TickBehavior), Added<TickBehavior>>,
) {
    for (interval, mut tick_behavior) in &mut area_timers {
        match *tick_behavior {
            TickBehavior::Static(ref mut static_timer) => {
                static_timer.set_duration(Duration::from_secs_f32(interval.0));
            }
            _ => (),
        }
    }
}

fn tick_hit_timers(
    time: Res<Time>,
    mut area_timers: Query<(
        &TargetsInArea,
        &mut Ticks,
        &FiringInterval,
        &mut TickBehavior,
        Option<&PausesWhenEmpty>,
        Option<&AreaTimeline>,
    )>,
) {
    for (targets_in_area, mut ticks, interval, mut tick_behavior, pauses, timeline) in
        &mut area_timers
    {
        // only tick area timers if has timeline and is firing
        if let Some(timeline) = timeline {
            if timeline.stage != DeployAreaStage::Firing {
                continue;
            }
        }
        match *tick_behavior {
            TickBehavior::Individual(ref mut individual_timers) => {
                // tick per-target timer and retain it
                individual_timers.map.retain(|_entity, hittimer| {
                    hittimer.tick(time.delta());
                    true
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

#[derive(Component)]
pub struct Fountain;

pub mod homing;
pub mod non_damaging;
