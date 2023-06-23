
use std::{collections::{HashMap}, time::{Duration, Instant}, cmp::Ordering};

use bevy_rapier3d::prelude::{Velocity, RigidBody, Sensor};
use derive_more::Display;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::Actionlike;
use rand::Rng;

use crate::{crowd_control::{CCInfo}, buff::{BuffInfo, BuffTargets}, game_manager::{Team, CastEvent}};
use shape::AbilityShape;
use homing::track_homing;

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AbilityInfo>();
        app.register_type::<Tags>();
        app.register_type::<CastingLifetime>();
        app.register_type::<EffectApplyType>();
        app.register_type::<TargetsInArea>();
        app.register_type::<TargetsToEffect>();

        app.add_event::<HealthChangeEvent>();
        app.add_event::<BuffEvent>();  
        app.add_event::<CCEvent>();   
        
        app.add_systems((
            tick_lifetime,
            tick_effect_types,
            track_homing,
        ));
        app.add_systems((
                catch_collisions,
                area_queue_targets,
                filter_targets,
                area_apply_tags,
                despawn_after_max_hits,
        ).chain());
    }
}


#[derive(Actionlike, Component, Reflect, FromReflect, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum Ability {
    Frostbolt,
    Fireball,
    #[default]
    BasicAttack,
    Dash,
}


impl Ability{
    // cooldown IS BEING USED
    pub fn get_cooldown(&self) -> f32{
        match self{
            Ability::Dash => 7.,
            Ability::Frostbolt => 3.5,
            Ability::Fireball => 4.,
            Ability::BasicAttack => 2.,
        }
    }


    // ISNT BEING USED, CLEAN UP OR REFACTOR
    // Cant just return bundle cus match arms are different tuples, need to pass in commands
    pub fn fire(&self, commands: &mut Commands) -> Entity{
        match self {
            Ability::Frostbolt => {
                commands.spawn((            
                    SpatialBundle::from_transform(
                        Transform {
                            translation: Vec3::new(0.0, 0.5, 0.0),
                            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                            ..default()
                    }),
                    Velocity {
                        linvel: Vec3::new(0.,0.,-1.) * 10.0,
                        ..default()
                    },
                    RigidBody::KinematicVelocityBased,
                    AbilityShape::Rectangle{
                        length: 1.0,
                        width: 1.0,
                    },
                    Sensor,
                    CastingLifetime { seconds: 1.0 },
                    Name::new("Frostbolt"),                
                )).id()
            },


            _ => {
                commands.spawn((     
                    SpatialBundle::from_transform(
                        Transform {
                            translation: Vec3::new(0.0, 0.5, 0.0),
                            ..default()
                    }),
                    AbilityShape::default(),
                    CastingLifetime { seconds: 2.0 },
                    Sensor,
                    Name::new("Ability Spawn"),  
                )).id()
            },
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
    pub fn new(ability: &Ability) -> Self{
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
pub struct TargetsInArea{
    pub list: Vec<Entity>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetsToEffect{
    pub list: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct TargetsHit{
    max: u8,
    current: u8,
}

impl TargetsHit{
    pub fn new(max: u8) -> Self{
        Self{
            max,
            current: 0,
        }
    }
}

impl Default for TargetsHit{
    fn default() -> Self {
        Self{
            max: 1,
            current:0,
        }
    }
}

// Probably need to combine this with QueueTargets and Tags so that we dont need to filter in last step
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub enum EffectApplyType{
    OnEnter(OnEnterEffect),
    Scan(ScanEffect),
}

impl Default for EffectApplyType{
    fn default() -> Self {
        Self::OnEnter(OnEnterEffect::default())
    }
}

#[derive(Debug, Clone, Default, Reflect, FromReflect)]
pub struct OnEnterEffect{
    pub target_penetration: u32,
    pub ticks: Ticks,
    pub hittimers: LastHitTimers,
}

#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct ScanEffect{
    pub ticks: Ticks,
    pub timer: Timer,
}

impl Default for ScanEffect{
    fn default() -> Self {
        let time_between_ticks = 1500;
        Self{
            ticks: Ticks::Multiple{
                amount: 8,
                interval: time_between_ticks
            },   
            timer: Timer::new(
                Duration::from_millis(time_between_ticks as u64),
                TimerMode::Repeating,
            ),     
        }
    }
}

#[derive(Debug, Clone, Default, Reflect, FromReflect)]
pub enum Ticks{
    #[default]
    Once,
    Multiple{
        amount: u32,        
        interval: u32, // in milliseconds
    },
    Unlimited{
        interval: u32,
    }
}

impl Ticks{
    fn get_interval(&self) -> u32{
        match self{
            Self::Once => 5000000, // just dont want it to hit same target twice, do something else later maybe
            Self::Multiple { amount: _, interval } => interval.clone(),
            Self::Unlimited { interval } => interval.clone(),
        }
    }
    fn get_amount(&self) -> u32 {
        match self{
            Self::Multiple{amount, interval: _} => *amount,
            _ => 1,
        }
    }
    fn set_amount(&mut self, new_amount: u32) {        
        match self{
            Self::Multiple{amount, interval: _} => {
                *amount = new_amount;
            }
            _ => (),
        }
    }
}

// map that holds what things have been hit by an ability instance
#[derive(Default, Debug, Clone, FromReflect, Reflect,)]
pub struct LastHitTimers {
    pub map: HashMap<Entity, Timer>,
}

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct FilterTargets{
    pub target_selection: TargetSelection,
    pub number_of_targets: u8,
}

#[derive(Default, Debug, Clone, FromReflect, Reflect,)]
pub enum TargetSelection{
    #[default]
    Closest,
    Random,
    HighestThreat, // Make Threat hashmap of <(Entity, Threat)>
    All,
}

pub struct HealthChangeEvent{
    pub amount: f32,
    pub attacker: Entity,
    pub defender: Entity,
    pub when: Instant,
}

pub struct BuffEvent{
    pub info: BuffInfo,
    pub target: Entity,
    pub buff_originator: Entity,
    pub caster: Entity,
    pub ability: Ability,
}

pub struct CCEvent{
    pub target_entity: Entity,
    pub ccinfo: CCInfo,
}

fn despawn_after_max_hits(
    mut commands: Commands,
    max_hits_query: Query<(Entity, &TargetsHit)>,
){
    for (entity, max_targets_hit) in max_hits_query.iter(){
        if max_targets_hit.current >= max_targets_hit.max{
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn area_apply_tags(
    mut sensor_query: Query<(Entity, &Team, &mut TargetsToEffect, &Tags, Option<&mut TargetsHit>)>,
    mut targets_query: Query<(Entity, &Team, Option<&Ability>)>,
    //rapier_context: Res<RapierContext>,
    mut health_events: EventWriter<HealthChangeEvent>,
    mut buff_events: EventWriter<BuffEvent>,
    mut cc_events: EventWriter<CCEvent>,
    mut cast_events: EventWriter<CastEvent>,
){
    for (sensor_entity, sensor_team, mut targets_to_effect, tags, mut max_targets_hit) in sensor_query.iter_mut(){
        for target_entity in targets_to_effect.list.iter_mut(){
            // double check they are intersecting, maybe dont even need since we are detecting enter and exit but im scared lol
            //if rapier_context.intersection_pair(sensor_entity, target_entity.clone()) == Some(true) {
            let Ok((_, target_team, ability)) = targets_query.get_mut(*target_entity) else { continue };
            let ability = ability.unwrap_or(&Ability::BasicAttack);
            let on_same_team = sensor_team.0 == target_team.0;

            let mut hit_a_target = false;
            for taginfo in tags.list.iter(){
                match taginfo {
                    TagInfo::Heal(amount) => {                                
                        if on_same_team{               
                            hit_a_target = true; 
                            let health_change = HealthChangeEvent{
                                amount: *amount,
                                attacker: sensor_entity,
                                defender: *target_entity,
                                when: Instant::now(),
                            };
                            health_events.send(health_change);  
                        }
                    }
                    TagInfo::Damage(amount) => {
                        if !on_same_team{     
                            hit_a_target = true; 
                            let health_change = HealthChangeEvent{
                                amount: -amount,
                                attacker: sensor_entity,
                                defender: *target_entity,
                                when: Instant::now(),
                            };
                            health_events.send(health_change);
                        }
                    }
                    TagInfo::Buff(ref buffinfo) => {
                        let buffing_ally = (buffinfo.bufftargets == BuffTargets::Allies) && on_same_team;
                        let debuffing_enemy = (buffinfo.bufftargets == BuffTargets::Enemies) && !on_same_team;
                        let buffing_anyone = buffinfo.bufftargets == BuffTargets::All;
                        
                        let buff_to_send = BuffEvent {
                            info: buffinfo.clone(),
                            ability: *ability,
                            buff_originator: sensor_entity, // TODO can be a specific ability, or an item
                            caster: sensor_entity,
                            target: *target_entity,
                        };
                        if buffing_ally || debuffing_enemy || buffing_anyone {                            
                            hit_a_target = true; 
                            buff_events.send(buff_to_send);
                        } 

                    }
                    TagInfo::CC(ref ccinfo) => {                           
                        hit_a_target = true; 
                        cc_events.send(CCEvent{
                            target_entity: *target_entity,
                            ccinfo: ccinfo.clone(),
                        });
                    }
                    TagInfo::Homing(ref projectile_to_spawn) => { 
                        hit_a_target = true;                                
                        cast_events.send(CastEvent {
                            caster: sensor_entity,
                            ability: projectile_to_spawn.clone(),
                        });
                    }
                }
            }
            if let Some(ref mut max_hits) = max_targets_hit{
                if hit_a_target{
                    max_hits.current += 1;
                }
                if max_hits.current >= max_hits.max{
                    continue;
                }
            }
        }
    }
}


fn filter_targets(
    mut sensor_query: Query<(&FilterTargets, &mut TargetsToEffect, &GlobalTransform)>,
    target_query: Query<&GlobalTransform> // add threat stat later 
){
    for (effect_apply_targets, mut targets_to_effect, sensor_transform ) in sensor_query.iter_mut(){
        if targets_to_effect.list.is_empty(){ continue }
        let mut new_targets: Vec<Entity> = Vec::new();
        match effect_apply_targets.target_selection{
            TargetSelection::Closest => {
                let num_of_targets = effect_apply_targets.number_of_targets;

                let mut closest_targets: Vec<(f32, Entity)> = Vec::new();
                for target_entity in targets_to_effect.list.iter(){
                    let Ok(target_transform) = target_query.get(*target_entity) else {continue};
                    let relative_translation = target_transform.translation() - sensor_transform.translation();
                    closest_targets.push((relative_translation.length(), *target_entity));
                }
                closest_targets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less));
                
                let (_, closest_entities): (Vec<_>, Vec<_>) = closest_targets.into_iter()
                    .take(num_of_targets as usize)
                    .unzip();
                new_targets = closest_entities;
            },
            TargetSelection::Random => {
                // can hit the same target twice lol, should remove from array and gen again but idc
                let mut rng = rand::thread_rng();
                for _ in 0..effect_apply_targets.number_of_targets{
                    let random_target_index = rng.gen_range(0..targets_to_effect.list.len());
                    let from_list = targets_to_effect.list.get(random_target_index).unwrap();
                    new_targets.push(*from_list);
                }
            },
            _ => (),
        }
        targets_to_effect.list = new_targets;
    }
}

fn area_queue_targets(
    mut sensor_query: Query<(&mut EffectApplyType, &TargetsInArea, &mut TargetsToEffect)>,
){
    for (mut effect_apply_type, targets_in_area, mut targets_to_effect) in sensor_query.iter_mut(){
        targets_to_effect.list = Vec::new();
        match *effect_apply_type{
            EffectApplyType::Scan(ref scaninfo) => {
                if !scaninfo.timer.just_finished(){
                    continue;
                }
                targets_to_effect.list = targets_in_area.list.clone();
            }
            EffectApplyType::OnEnter(ref mut enterinfo) =>{
                for target_entity in targets_in_area.list.iter(){
                    let interval_over = match enterinfo.hittimers.map.get(&target_entity) {
                        Some(timer) => timer.just_finished(),
                        None => true,
                    };
                    if interval_over{
                        targets_to_effect.list.push(*target_entity);
                        enterinfo.hittimers.map.insert(
                            *target_entity,
                            Timer::new(
                                Duration::from_millis(enterinfo.ticks.get_interval() as u64),
                                TimerMode::Once,
                            ),
                        );
                    }
                }
            }
        }
        
    }
}

// Change to anything with damage component
// Add team support
// Add penetration and despawning (separate system?)
fn catch_collisions(
    targets_query: Query<Entity>,
    mut sensor_query: Query<(Entity, &mut TargetsInArea)>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        //dbg!(collision_event.clone());
        let (
            ( _area_entity, mut targets_in_area), 
            target_entity, 
            colliding
        ) =
            match collision_event{
                &CollisionEvent::Started(collider1, collider2, _flags) => {
                    let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(collider1){
                        (sensor, collider2)
                    } else if let Ok(sensor) = sensor_query.get_mut(collider2) {
                        (sensor, collider1)
                    } else{
                        continue;
                    };

                    if let Ok(target) = targets_query.get(potential){
                        (sensor, target, true)
                    } else{
                        continue;
                    }
                }
                &CollisionEvent::Stopped(collider1, collider2, _flags) => {
                    let (sensor, potential) = if let Ok(sensor) = sensor_query.get_mut(collider1){
                        (sensor, collider2)
                    } else if let Ok(sensor) = sensor_query.get_mut(collider2) {
                        (sensor, collider1)
                    } else{
                        continue;
                    };

                    if let Ok(target) = targets_query.get(potential){
                        (sensor, target, false)
                    } else{
                        continue;
                    }
                }
            };
        if colliding{
            targets_in_area.list.push(target_entity);
        } else{
            if let Some(index) = targets_in_area.list.iter().position(|x| *x == target_entity){
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
            commands.entity(entity).despawn();
        }
    }
}

fn tick_effect_types(time: Res<Time>, mut effect_timers: Query<&mut EffectApplyType>){
    for mut effect_type in &mut effect_timers {
        match *effect_type{
            EffectApplyType::OnEnter(ref mut enterinfo) => { // tick per-target timer and retain it if not finished
                enterinfo.hittimers.map.retain(|_entity, hittimer| {
                    hittimer.tick(time.delta());
                    !hittimer.finished() 
                });
            }
            EffectApplyType::Scan(ref mut scaninfo) => { // tick whole ability timer
                scaninfo.timer.tick(time.delta());
                match scaninfo.ticks {
                    Ticks::Once => scaninfo.timer.pause(),
                    Ticks::Multiple { amount: _, interval: _ } => {
                        if scaninfo.timer.just_finished(){
                            let new_amount = scaninfo.ticks.get_amount() - 1;
                            scaninfo.ticks.set_amount(new_amount);
                        }
                        if scaninfo.ticks.get_amount() == 0 {
                            scaninfo.timer.pause();
                        }
                    },
                    _ => (), // do nothing because timer is set to repeating for scan, works for unlimited ticks
                }
            }
        }
    }
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Tags{
    pub list: Vec<TagInfo>
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub enum TagInfo{
    Heal(f32),
    Damage(f32),
    Buff(BuffInfo),
    CC(CCInfo),
    Homing(Ability),
}


#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct HomingInfo{
    projectile_to_spawn: Ability,
}



pub mod ability_bundles;
pub mod shape;
pub mod homing;