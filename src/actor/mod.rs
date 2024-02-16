use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    actor::{
        bounty::{increment_bounty, Bounty},
        cast::CastPlugin,
        controller::*,
        log::{DamageSum, LogHit, LogSide, LogType},
        minion::MinionPlugin,
        player::*,
    },
    crowd_control::CCMap,
    prelude::*,
    session::director::InGameSet,
    stats::HealthMitigatedEvent,
    ui::scoreboard::Scoreboard,
};

pub mod bounty;
pub mod cast;
pub mod controller;
pub mod log;
pub mod minion;
pub mod player;
pub mod rank;

pub struct ActorPlugin;
impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Bounty>();
        //Resources
        app.add_event::<LogHit>();
        app.add_event::<KillEvent>();

        //Plugins
        app.add_plugins((MinionPlugin, ControllerPlugin, PlayerPlugin, CastPlugin));

        //Systems
        // Process transforms always after inputs, and translations after rotations
        app.add_systems(
            PostUpdate,
            (
                player_swivel,
                player_movement,
                give_kill_credit,
                start_death_animation,
            )
                .chain()
                .in_set(InGameSet::Post),
        );
        app.add_systems(
            FixedUpdate,
            (update_damage_logs, increment_bounty).in_set(InGameSet::Update),
        );
        app.add_systems(Last, (tick_death_animation).in_set(InGameSet::Update));
    }
}

#[derive(Component, Clone, Hash, PartialEq, Eq, Reflect)]
pub enum ActorType {
    Player(Player),
    Minion,
}

#[derive(Component, Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect)]
#[reflect(Component)]
pub enum ActorState {
    Alive,
    #[default]
    Dead,
}

impl ActorState {
    pub fn is_alive(&self) -> bool {
        self == &ActorState::Alive
    }
    pub fn is_dead(&self) -> bool {
        self == &ActorState::Dead
    }
}

#[derive(Component, Deref, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct PreviousActorState(ActorState);

pub fn update_previous_actor(mut actors: Query<(&mut PreviousActorState, &ActorState)>) {
    for (mut previous, current) in &mut actors {
        previous.0 = current.clone();
    }
}

#[derive(Component)]
pub struct HasHealthBar;

pub fn player_swivel(mut players: Query<(&mut Transform, &PlayerInput, &CCMap, &ActorState), With<Player>>) {
    for (mut player_transform, inputs, cc_map, state) in players.iter_mut() {
        if cc_map.is_stunned() || !state.is_alive() {
            continue
        }
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}

pub fn player_movement(
    mut query: Query<(
        &Attributes,
        &mut Controller,
        &PlayerInput,
        &CCMap,
        &ActorState,
    )>,
) {
    for (attributes, mut controller, player_input, cc_map, state) in query.iter_mut() {
        if cc_map.is_rooted() || cc_map.is_stunned() || !state.is_alive() {
            controller.direction = Vec3::ZERO;
            // need to set to zero otherwise once stunned you 'skate' in that direction
            continue
        }

        let speed = attributes.get(Stat::Speed);
        let mut direction = Vec3::new(0.0, 0.0, 0.0);
        if player_input.left() {
            direction.x += -1.;
        }
        if player_input.right() {
            direction.x += 1.;
        }
        if player_input.back() {
            direction.z += 1.;
        }
        if player_input.forward() {
            direction.z += -1.;
        }

        let direction_normalized = direction.normalize_or_zero();
        let movement_vector = Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32) * direction_normalized;

        controller.direction = movement_vector;
        controller.max_speed = speed;
    }
}

fn give_kill_credit(
    changed_states: Query<(Option<&Bounty>, &ActorState, &IncomingDamageLog, Entity), Changed<ActorState>>,
    mut victors: Query<(&mut Attributes, &ActorType)>,
    mut scoreboard: ResMut<Scoreboard>,
    mut kill_events: EventWriter<KillEvent>,
) {
    const TIME_FOR_KILL_CREDIT: u64 = 30;
    for (bounty, state, log, damned) in changed_states.iter() {
        if state.is_alive() {
            continue
        }

        let mut killers = Vec::new();
        for instance in log.list.iter().rev() {
            if Instant::now().duration_since(instance.when) > Duration::from_secs(TIME_FOR_KILL_CREDIT) {
                break
            }
            //let Ok(attacker) = the_guilty.get(instance.attacker) else {continue};
            killers.push(instance.attacker);
        }
        for (index, awardee) in killers.iter().enumerate() {
            let Ok((mut attributes, awardee_actor)) = victors.get_mut(*awardee) else { continue };

            if let Some(bounty) = bounty {
                let gold = attributes.get_mut(Stat::Gold);
                *gold += bounty.gold;
                let xp = attributes.get_mut(Stat::Xp);
                *xp += bounty.xp;
            }

            if let ActorType::Player(killer) = awardee_actor {
                let killer_scoreboard = scoreboard.0.entry(*killer).or_default();
                if index == 0 {
                    killer_scoreboard.kda.kills += 1;
                } else {
                    killer_scoreboard.kda.assists += 1;
                }
            }
        }
        let Some(killer) = killers.get(0) else { continue };
        kill_events.send(KillEvent {
            killer: *killer,
            accomplices: killers[1..].to_vec(),
            damned: damned,
        })
    }
}

#[derive(Event)]
pub struct KillEvent {
    pub killer: Entity,
    pub accomplices: Vec<Entity>,
    pub damned: Entity,
}

#[derive(Component)]
pub struct DespectateDelay(pub Timer);

#[derive(Component)]
pub struct DeathDelay(pub Timer);

fn start_death_animation(mut commands: Commands, changed_states: Query<(Entity, &ActorState), Changed<ActorState>>) {
    let delay_time = 3.0;
    for (entity, state) in changed_states.iter() {
        if state.is_alive() {
            continue
        }

        // Start death animation
        // Add despawn delay component to dead thing
        let death_delay = DeathDelay(Timer::from_seconds(delay_time, TimerMode::Once));
        commands.entity(entity).insert(death_delay);
    }
}

fn tick_death_animation(
    mut respawning: Query<(Entity, &mut DeathDelay, &mut Visibility)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer, mut vis) in respawning.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.percent() > 0.7 {
            *vis = Visibility::Hidden;
        }
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// probably want to change this respawn system to just despawn the entity instead of hiding + moving it
//
// it makes sense to just move players back to spawn at first, but what about minions?
// the hiding + moving way works for players cus u will have just 1
// obv we cant just keep adding minions, we def despawn them so i think having a unified way of de/respawning things too
// and should do the same for players, fully despawning and setting a respawn in something managed by the director instead of component
// fn respawn_entity(
//     mut commands: Commands,
//     mut the_damned: Query<(&mut Visibility, &ActorState, Entity, &ActorType), Changed<ActorState>>,
//     local_player: Res<Player>,
//     //mut spectate_events: EventWriter<SpectateEvent>,
// ) {
//     for (mut vis, state, entity, actor_type) in the_damned.iter_mut(){
//         if state == ActorState::Dead { continue }
//         commands.entity(entity).remove::<Respawn>();
//         *vis = Visibility::Visible;
//         if actor_type == ActorType::Player(*local_player) {
//             spectate_events.send(SpectateEvent {
//                 entity: event.entity,
//             });
//         }
//     }
// }

#[derive(Component, Default)]
pub struct OutgoingDamageLog {
    pub list: Vec<HealthMitigatedEvent>,
    pub sums: HashMap<Entity, HashMap<Entity, DamageSum>>,
}

#[derive(Component, Default)]
pub struct IncomingDamageLog {
    pub list: Vec<HealthMitigatedEvent>,
    pub sums: HashMap<Entity, DamageSum>,
}
// change mitigated function to round properly, dont need to cast to ints here
fn update_damage_logs(
    mut damage_events: EventReader<HealthMitigatedEvent>,
    mut incoming_logs: Query<&mut IncomingDamageLog>,
    mut outgoing_logs: Query<&mut OutgoingDamageLog>,
    mut log_hit_events: EventWriter<LogHit>,
) {
    for damage_instance in damage_events.read() {
        if let Ok(mut defender_log) = incoming_logs.get_mut(damage_instance.defender) {
            defender_log.list.push(damage_instance.clone());
            if defender_log.sums.contains_key(&damage_instance.sensor) {
                let Some(hits) = defender_log.sums.get_mut(&damage_instance.sensor) else { continue };
                hits.add_damage(damage_instance.clone());
                log_hit_events.send(LogHit::new(
                    damage_instance.clone(),
                    LogType::Stack,
                    LogSide::Incoming,
                ));
            } else {
                defender_log.sums.insert(
                    damage_instance.sensor.clone(),
                    DamageSum::from_instance(damage_instance.clone()),
                );
                log_hit_events.send(LogHit::new(
                    damage_instance.clone(),
                    LogType::Add,
                    LogSide::Incoming,
                ));
            }
        }

        if let Ok(mut attacker_log) = outgoing_logs.get_mut(damage_instance.attacker) {
            attacker_log.list.push(damage_instance.clone());
            if attacker_log.sums.contains_key(&damage_instance.sensor) {
                let Some(targets_hit) = attacker_log.sums.get_mut(&damage_instance.sensor) else { continue };
                if targets_hit.contains_key(&damage_instance.defender) {
                    let Some(hits) = targets_hit.get_mut(&damage_instance.defender) else { continue };
                    hits.add_damage(damage_instance.clone());
                    log_hit_events.send(LogHit::new(
                        damage_instance.clone(),
                        LogType::Stack,
                        LogSide::Outgoing,
                    ));
                } else {
                    targets_hit.insert(
                        damage_instance.defender.clone(),
                        DamageSum::from_instance(damage_instance.clone()),
                    );
                    log_hit_events.send(LogHit::new(
                        damage_instance.clone(),
                        LogType::Add,
                        LogSide::Outgoing,
                    ));
                }
            } else {
                let init = HashMap::from([(
                    damage_instance.defender,
                    DamageSum::from_instance(damage_instance.clone()),
                )]);
                attacker_log.sums.insert(damage_instance.sensor.clone(), init);
                log_hit_events.send(LogHit::new(
                    damage_instance.clone(),
                    LogType::Add,
                    LogSide::Outgoing,
                ));
            }
        }
    }
}
