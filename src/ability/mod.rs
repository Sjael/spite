use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::{RigidBody, Sensor, Velocity};
use derive_more::Display;
use leafwing_input_manager::Actionlike;
use rand::Rng;

use crate::{
    buff::{BuffInfo, BuffTargets},
    crowd_control::CCInfo,
    game_manager::{Team, FireHomingEvent},
};
use homing::track_homing;
use shape::AbilityShape;

use self::ability_bundles::Caster;

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AbilityInfo>();
        app.register_type::<CastingLifetime>();
        app.register_type::<TickBehavior>();
        app.register_type::<TargetsInArea>();
        app.register_type::<TargetsHittable>();

        app.add_event::<HealthChangeEvent>();
        app.add_event::<BuffEvent>();
        app.add_event::<CCEvent>();

        app.add_systems((tick_lifetime, tick_hit_timers, track_homing, apply_interval));
        app.add_systems(
            (
                catch_collisions,
                filter_targets,
                area_queue_targets,
                area_apply_tags,
                despawn_after_max_hits,
            )
                .chain(),
        );
    }
}

#[derive(
    Actionlike,
    Component,
    Reflect,
    FromReflect,
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Hash,
)]
#[reflect(Component)]
pub enum Ability {
    Frostbolt,
    Fireball,
    #[default]
    BasicAttack,
    Dash,
}

impl Ability {
    // cooldown IS BEING USED
    pub fn get_cooldown(&self) -> f32 {
        match self {
            Ability::Dash => 7.,
            Ability::Frostbolt => 3.5,
            Ability::Fireball => 4.,
            Ability::BasicAttack => 2.,
        }
    }

    pub fn get_windup(&self) -> f32{
        match self {
            Ability::Frostbolt => 0.2,
            Ability::Fireball => 0.8,
            _ => 0.4,
        }
    }

    // ISNT BEING USED, CLEAN UP OR REFACTOR
    // Cant just return bundle cus match arms are different tuples, need to pass in commands
    pub fn fire(&self, commands: &mut Commands) -> Entity {
        match self {
            Ability::Frostbolt => commands
                .spawn((
                    SpatialBundle::from_transform(Transform {
                        translation: Vec3::new(0.0, 0.5, 0.0),
                        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                        ..default()
                    }),
                    Velocity {
                        linvel: Vec3::new(0., 0., -1.) * 10.0,
                        ..default()
                    },
                    RigidBody::KinematicVelocityBased,
                    AbilityShape::Rectangle {
                        length: 1.0,
                        width: 1.0,
                    },
                    Sensor,
                    CastingLifetime { seconds: 1.0 },
                    Name::new("Frostbolt"),
                ))
                .id(),

            _ => commands
                .spawn((
                    SpatialBundle::from_transform(Transform {
                        translation: Vec3::new(0.0, 0.5, 0.0),
                        ..default()
                    }),
                    AbilityShape::default(),
                    CastingLifetime { seconds: 2.0 },
                    Sensor,
                    Name::new("Ability Spawn"),
                ))
                .id(),
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AbilityInfo {
    pub title: String,
    pub image: String,
    pub description: String,
}

impl AbilityInfo {
    pub fn new(ability: &Ability) -> Self {
        match ability {
            &Ability::Frostbolt => AbilityInfo {
                title: "Frostbolt".to_string(),
                image: "icons/frostbolt.png".to_string(),
                description: "Cold as fuck".to_string(),
            },
            &Ability::Dash => AbilityInfo {
                title: "Driving Strike".to_string(),
                image: "icons/dash.png".to_string(),
                description: "Hercules delivers a mighty strike, driving all enemies back, damaging and Stunning them. Hercules is immune to Knockback during the dash.".to_string(),
            },
            _ => AbilityInfo {
                title: "Ability".to_string(),
                image: "icons/BasicAttack.png".to_string(),
                description: "A very boring attack".to_string(),
            },
        }
    }
}

// Maybe we should have a "general" lifetime too even if the thing isn't being casted...?
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct CastingLifetime {
    pub seconds: f64,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsInArea {
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct TargetFilter {
    pub target_selection: TargetSelection,
    pub number_of_targets: u8,
}

#[derive(Default, Debug, Clone, FromReflect, Reflect)]
pub enum TargetSelection {
    #[default]
    Closest,
    Random,
    HighestThreat, // Make Threat hashmap of <(Entity, Threat)>
    All,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FilteredTargets {
    pub list: Vec<Entity>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsHittable {
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct MaxTargetsHit {
    max: u8,
    current: u8,
}

impl MaxTargetsHit {
    pub fn new(max: u8) -> Self {
        Self { max, current: 0 }
    }
}

impl Default for MaxTargetsHit {
    fn default() -> Self {
        Self { max: 255, current: 0 }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct UniqueTargetsHit {
    already_hit: Vec<Entity>,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub enum TickBehavior{
    Static(Timer),
    Individual(IndividualTargetTimers),
}

impl Default for TickBehavior{
    fn default() -> Self {
        Self::Static(Timer::default())
    }
}

impl TickBehavior{
    pub fn static_timer() -> Self{
        Self::Static(Timer::new(Duration::from_secs(1), TimerMode::Repeating))    
    }
    pub fn individual() -> Self{
        Self::Individual(IndividualTargetTimers::default())
    }
}

#[derive(Default, Reflect, FromReflect, Debug, Clone)]
pub struct StaticTimer(pub Timer);

// map that holds what things have been hit by an ability instance
#[derive(Default, Debug, Clone, FromReflect, Reflect)]
pub struct IndividualTargetTimers {
    pub map: HashMap<Entity, Timer>,
}

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub struct PausesWhenEmpty;

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub enum Ticks {
    #[default]
    Once,
    Multiple(u32),
    Unlimited,
}

#[derive(Component, Debug, Clone, Default, Reflect, FromReflect)]
pub struct FiringInterval(pub u32);

pub struct HealthChangeEvent {
    pub amount: f32,
    pub attacker: Option<Entity>,
    pub defender: Entity,
    pub when: Instant,
}

pub struct BuffEvent {
    pub info: BuffInfo,
    pub target: Entity,
    pub buff_originator: Entity,
    pub caster: Option<Entity>,
    pub ability: Ability,
}

pub struct CCEvent {
    pub target_entity: Entity,
    pub ccinfo: CCInfo,
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
        Option<&Caster>,
        Option<&FiringInterval>,
        Option<&mut MaxTargetsHit>,
        Option<&mut TickBehavior>,
        Option<&mut UniqueTargetsHit>,
    )>,
    mut targets_query: Query<(Entity, &Team, Option<&Ability>)>,
    mut health_events: EventWriter<HealthChangeEvent>,
    mut buff_events: EventWriter<BuffEvent>,
    mut cc_events: EventWriter<CCEvent>,
    //mut cast_events: EventWriter<CastEvent>,
    mut cast_homing_events: EventWriter<FireHomingEvent>,
) {
    for (sensor_entity, sensor_team, targets_hittable, tags, caster, interval,
        mut max_targets_hit, mut tick_behavior, mut unique_targets_hit) in &mut sensor_query
    {        
        let mut targets_that_got_hit: Vec<Entity> = Vec::new();
        for target_entity in targets_hittable.list.iter() {
            let Ok((_, target_team, ability)) = targets_query.get_mut(*target_entity) else { continue };
            let ability = ability.unwrap_or(&Ability::BasicAttack);
            let caster = if let Some(caster) = caster{
                Some(caster.0)
            }else {
                None
            };
            let on_same_team = sensor_team.0 == target_team.0;
            if let Some(ref unique_targets_hit) = unique_targets_hit{
                if unique_targets_hit.already_hit.contains(target_entity){ continue }
            }

            let mut hit_a_target = false;
            for taginfo in tags.list.iter() {
                match taginfo {
                    TagInfo::Heal(amount) => {
                        if on_same_team {
                            hit_a_target = true;
                            let health_change = HealthChangeEvent {
                                amount: *amount,
                                attacker: caster,
                                defender: *target_entity,
                                when: Instant::now(),
                            };
                            health_events.send(health_change);
                        }
                    }
                    TagInfo::Damage(amount) => {
                        if !on_same_team {
                            println!("damage is {}", amount);
                            hit_a_target = true;
                            let health_change = HealthChangeEvent {
                                amount: -amount,
                                attacker: caster,
                                defender: *target_entity,
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
                        if !on_same_team{
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
                            caster: sensor_entity,
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
                        return
                    }
                }
            }
        }
        if let Some(mut tick_behavior) = tick_behavior{
            if let Some(interval) = interval{
                if let TickBehavior::Individual(ref mut individual_timers) = *tick_behavior{
                    for got_hit in targets_that_got_hit.iter(){
                        individual_timers.map.insert(
                            *got_hit,
                            Timer::new(
                                Duration::from_millis(interval.0 as u64),
                                TimerMode::Once,
                            ),
                        );
                    }                        
                }
            }
        }
        for got_hit in targets_that_got_hit.iter(){
            if let Some(ref mut unique_targets_hit) = unique_targets_hit{
                unique_targets_hit.already_hit.push(*got_hit);
            }        
        }
    }
}


fn area_queue_targets(
    mut sensor_query: Query<(&TargetsInArea, &mut TargetsHittable, Option<&TickBehavior>, Option<&FilteredTargets>)>,
) {
    for (targets_in_area, mut targets_hittable,  
        tick_behavior, filtered_targets) in sensor_query.iter_mut() {
        targets_hittable.list = Vec::new();
        if let Some(tick_behavior) = tick_behavior {
            match *tick_behavior {
                TickBehavior::Static(ref static_timer) => {
                    if !static_timer.finished() {
                        continue;
                    }
                    if let Some(filtered_targets) = filtered_targets {
                        targets_hittable.list = filtered_targets.list.clone();
                    } else{
                        targets_hittable.list = targets_in_area.list.clone();
                    }
                }
                TickBehavior::Individual(ref individual_timers) => {
                    for target_entity in targets_in_area.list.iter() {
                        if let Some(filtered_targets) = filtered_targets {
                            if !filtered_targets.list.contains(target_entity){
                                continue
                            }
                        }
                        let hasnt_been_hit_or_interval_over = match individual_timers.map.get(&target_entity) {
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
            targets_hittable.list = targets_in_area.list.clone();
        }
    }
}

fn filter_targets(
    mut sensor_query: Query<(&TargetFilter, &GlobalTransform, &TargetsInArea, &mut FilteredTargets, Entity)>,
    changed_sensors: Query<Entity, Changed<TargetsInArea>>,
    target_query: Query<&GlobalTransform>, // add threat stat later
) {
    for (target_filter, sensor_transform,
    targets_in_area, mut filtered_targets, sensor_entity) in sensor_query.iter_mut() {
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
                    let Ok(target_transform) = target_query.get(*target_entity) else {continue};
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
                let Ok(_) = changed_sensors.get(sensor_entity) else { continue };
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
fn catch_collisions(
    targets_query: Query<Entity, Without<Sensor>>,
    mut sensor_query: Query<(Entity, &mut TargetsInArea), With<Sensor>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        let ((_area_entity, mut targets_in_area), target_entity, colliding) = match collision_event {
            &CollisionEvent::Started(collider1, collider2, _flags) => {
                let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(collider1) {
                    (sensor, collider2)
                } else if let Ok(sensor) = sensor_query.get_mut(collider2) {
                    (sensor, collider1)
                } else {
                    continue;
                };

                if let Ok(target) = targets_query.get(potential) {
                    (sensor, target, true)
                } else {
                    continue;
                }
            }
            &CollisionEvent::Stopped(collider1, collider2, _flags) => {
                let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(collider1) {
                    (sensor, collider2)
                } else if let Ok(sensor) = sensor_query.get_mut(collider2) {
                    (sensor, collider1)
                } else {
                    continue;
                };

                if let Ok(target) = targets_query.get(potential) {
                    (sensor, target, false)
                } else {
                    continue;
                }
            }
        };
        if colliding {
            targets_in_area.list.push(target_entity);
        } else {
            if let Some(index) = targets_in_area
                .list
                .iter()
                .position(|x| *x == target_entity)
            {
                targets_in_area.list.remove(index);
            }
        }
    }
}

fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut CastingLifetime, Entity)>,
) {
    for (mut lifetime, entity) in lifetimes.iter_mut() {
        //dbg!(lifetime.clone());
        lifetime.seconds -= time.delta_seconds_f64();
        if lifetime.seconds <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn apply_interval(
    mut area_timers: Query<(&FiringInterval, &mut TickBehavior), Added<TickBehavior>>
){
    for (interval, mut tick_behavior) in &mut area_timers{
        match *tick_behavior{
            TickBehavior::Static(ref mut static_timer) => {
                static_timer.set_duration(Duration::from_millis(interval.0 as u64));
            }
            _ => ()
        }
    }
}

fn tick_hit_timers(
    time: Res<Time>, 
    mut area_timers: Query<(&mut Ticks, &FiringInterval, &TargetsInArea, &mut TickBehavior, Option<&PausesWhenEmpty>)>
) {
    for (mut ticks, interval,  targets_in_area, 
    mut tick_behavior, pauses) in &mut area_timers {
        
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
                static_timer.tick(time.delta());
                if pauses.is_some() && targets_in_area.list.is_empty(){ 
                    static_timer.set_mode(TimerMode::Once);
                } else if static_timer.mode() == TimerMode::Once{
                    static_timer.set_mode(TimerMode::Repeating);
                }
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

#[derive(Component, Default, Clone, Debug)]
pub struct Tags {
    pub list: Vec<TagInfo>,
}

#[derive(Clone, Debug)]
pub enum TagInfo {
    Heal(f32),
    Damage(f32),
    Buff(BuffInfo),
    CC(CCInfo),
    Homing(Ability),
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct HomingInfo {
    projectile_to_spawn: Ability,
}

pub mod ability_bundles;
pub mod homing;
pub mod shape;
