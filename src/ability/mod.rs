
use std::{collections::{HashMap, VecDeque}, time::{Duration, Instant}, cmp::Ordering};

use bevy_rapier3d::prelude::{Velocity, RigidBody, Sensor};
use derive_more::Display;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::Actionlike;
use rand::Rng;

use crate::{stats::{Health, Attribute}, crowd_control::{CCInfo}, buff::{BuffInfo, BuffInfoTest}, player::{BuffMap, CCMap, IncomingDamageLog}, game_manager::{Team, CastEvent}};
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

        app.add_event::<DamageInstance>();
        app.add_event::<BuffEvent>();
        
        app.add_systems((
            tick_lifetime,
            tick_effect_types,
            track_homing,
        ));
        app.add_systems((
                catch_collisions,
                area_queue_targets,
                filter_targets,
                area_apply_effects,
                despawn_after_max_hits,
        ).chain());
    }
}


#[derive(Actionlike, Component, Reflect, FromReflect, Clone, Debug, Default, Display, Eq, PartialEq, Hash)]
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
    current: u8,
    max: u8,
}

impl TargetsHit{
    pub fn new(max: u8) -> Self{
        TargetsHit{
            max,
            ..default() 
        }
    }
}

impl Default for TargetsHit{
    fn default() -> Self {
        Self{
            current:0,
            max: 1,
        }
    }
}

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
    // how many targets it can hit before despawning
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

pub struct DamageInstance{
    pub damage: u32,
    pub mitigated: u32,
    pub attacker: Entity,
    pub defender: Entity,
    pub when: Instant,
}

pub struct BuffEvent{
    pub info: BuffInfoTest,
    pub entity: Entity,
    pub team: Team,
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

fn area_apply_effects(
    mut sensor_query: Query<(Entity, &mut TargetsToEffect, &Tags, Option<&mut TargetsHit>)>,
    mut targets_query: Query<(Entity, &mut Attribute<Health>, &Team, &mut BuffMap, &mut CCMap, &mut IncomingDamageLog)>,
    //rapier_context: Res<RapierContext>,
    mut damage_events: EventWriter<DamageInstance>,
    mut buff_events: EventWriter<BuffEvent>,
    mut cast_events: EventWriter<CastEvent>,
){
    for (sensor_entity, mut targets_to_effect, tags, mut max_targets_hit) in sensor_query.iter_mut(){
        for target_entity in targets_to_effect.list.iter_mut(){
            // double check they are intersecting, maybe dont even need since we are detecting enter and exit but im scared lol
            //if rapier_context.intersection_pair(sensor_entity, target_entity.clone()) == Some(true) {
            let Ok((_, mut health, target_team, mut buffs, mut cc, mut inc_damage_log)) 
                = targets_query.get_mut(*target_entity) else { continue};
            
            if let Some(ref mut max_hits) = max_targets_hit{
                max_hits.current += 1;
                if max_hits.current > max_hits.max{
                    continue;
                }
            } 

            for taginfo in tags.list.iter(){
                let on_same_team = taginfo.team.0 == target_team.0;
                // change to send events
                match taginfo.tag {
                    TagType::Heal(heal) => {                                
                        if on_same_team{                                                         
                            println!("healing is {:?}", heal);
                            let new_hp = health.amount() + heal;
                            health.set_amount(new_hp);
                        }
                    }
                    TagType::Damage(damage) => {
                        if !on_same_team{
                            println!("damage is {:?}", damage);
                            let new_hp = health.amount() - damage; //  send these to mitigation handler
                            health.set_amount(new_hp);
                            let damage_instance = DamageInstance{
                                damage: damage as u32,
                                mitigated: 0,
                                attacker: sensor_entity,
                                defender: *target_entity,
                                when: Instant::now(),
                            };
                            damage_events.send(damage_instance);
                            //inc_damage_log.map.push(damage_instance);
                        }
                    }
                    TagType::Buff(ref buffinfo) => {
                        buff_events.send(BuffEvent {
                            info: buffinfo.clone(),
                            entity: *target_entity,
                            team: taginfo.team,
                        });
                        dbg!(buffinfo.clone());
                    }
                    TagType::CC(ref ccinfo) => {
                        cc.map.insert(
                            ccinfo.cctype.clone(),
                            Timer::new(Duration::from_millis((ccinfo.duration * 1000.0) as u64), TimerMode::Once), 
                        );
                    }
                    TagType::Homing(ref projectile_to_spawn) => {                                
                        cast_events.send(CastEvent {
                            caster: sensor_entity,
                            ability: projectile_to_spawn.clone(),
                        });
                    }
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
    mut sensor_query: Query<(&mut EffectApplyType, &mut TargetsInArea, &mut TargetsToEffect)>,
){
    for (mut effect_apply_type, mut targets_in_area, mut targets_to_effect) in sensor_query.iter_mut(){
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

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct TagInfo{
    pub tag:TagType,
    pub team: Team,
}

#[derive(Clone, Debug, Reflect, FromReflect)]
pub enum TagType{
    CC(CCInfo),
    Damage(f32),
    Buff(BuffInfoTest),
    Heal(f32),
    Homing(Ability),
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct HomingInfo{
    projectile_to_spawn: Ability,
}


impl Default for TagType{
    fn default() -> Self {
        Self::Damage(10.0)
    }
}


pub mod ability_bundles;
pub mod shape;
pub mod homing;