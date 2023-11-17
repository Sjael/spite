use std::{collections::HashMap, time::Duration};

use crate::{
    ability::{rank::Rank, Ability, DamageType},
    actor::view::{Spectatable, SpectateEvent, Spectating},
    game_manager::{AbilityFireEvent, Bounty, InGameSet, PLAYER_LAYER, TEAM_1},
    input::{copy_action_state, SlotBundle},
    inventory::Inventory,
    prelude::*,
    ui::{
        store::{StoreBuffer, StoreHistory},
        Trackable,
    },
    GameState,
};
use oxidized_navigation::NavMeshAffector;

use self::{
    buff::{BuffMap, BuffPlugin},
    crowd_control::{CCMap, CCPlugin, CCType},
    minion::MinionPlugin,
    player::*,
    stats::{AttributeTag, Attributes, HealthMitigatedEvent, Modifier, Stat, StatsPlugin},
};

pub mod buff;
pub mod crowd_control;
pub mod minion;
pub mod player;
pub mod stats;
pub mod view;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        //Resources
        app.insert_resource(PlayerInput::default());
        app.register_type::<PlayerInput>()
            .register_type::<CooldownMap>()
            .register_type::<HoveredAbility>()
            .register_type::<Attributes>()
            .register_type::<Stat>()
            .register_type::<Modifier>()
            .register_type::<AttributeTag>();

        app.add_event::<InputCastEvent>()
            .add_event::<CastEvent>()
            .add_event::<InitSpawnEvent>()
            .add_event::<RespawnEvent>()
            .add_event::<LogHit>();

        //Plugins
        app.add_plugins((StatsPlugin, BuffPlugin, CCPlugin, MinionPlugin));

        //Systems
        app.add_systems(OnEnter(GameState::InGame), setup_player);
        app.add_systems(
            PreUpdate,
            (
                player_keys_input,
                player_mouse_input,
                select_ability.after(copy_action_state),
                respawn_entity,
                update_local_player_inputs,
            )
                .in_set(InGameSet::Pre),
        );
        app.add_systems(
            Update,
            (
                cast_ability,
                normal_casting,
                show_targetter.after(normal_casting),
                change_targetter_color,
                trigger_cooldown.after(cast_ability),
                tick_cooldowns.after(trigger_cooldown),
                start_ability_windup.after(cast_ability),
                tick_windup_timer,
                init_player,
                update_damage_logs,
            )
                .in_set(InGameSet::Update),
        );
        // Process transforms always after inputs, and translations after rotations
        app.add_systems(
            PostUpdate,
            (player_swivel, player_movement)
                .chain()
                .in_set(InGameSet::Post),
        );
    }
}

pub struct ActorInfo {
    pub entity: Entity,
    pub actor: ActorType,
}

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub enum ActorType {
    Player(Player),
    Minion,
}

#[derive(Component, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum ActorState {
    Alive,
    #[default]
    Dead,
}

#[derive(Component)]
pub struct HasHealthBar;

#[derive(Event)]
pub struct InitSpawnEvent {
    pub actor: ActorType,
    pub transform: Transform,
}

#[derive(Event)]
pub struct RespawnEvent {
    pub entity: Entity,
    pub actor: ActorType,
}

fn init_player(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut spawn_events: EventReader<InitSpawnEvent>,
    mut spectate_events: EventWriter<SpectateEvent>,
    local_player: Res<Player>,
) {
    for event in spawn_events.read() {
        let player = match event.actor {
            ActorType::Player(player) => player,
            _ => continue,
        };
        let spawning_id = player.id.clone();
        info!("spawning player {}", spawning_id);
        // reset the rotation so you dont spawn looking the other way

        let player_entity = commands
            .spawn(SpatialBundle::from_transform(event.transform.clone()))
            .insert((
                event.actor.clone(), // ActorType
                player,              // Player
                Name::new(format!("Player {}", spawning_id.to_string())),
                ActorState::Alive,
            ))
            /*.insert(ControllerBundle {
                controller: Controller {
                    float: Float {
                        spring: Spring {
                            strength: SpringStrength::AngularFrequency(25.0),
                            damping: 0.8,
                        },
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            */
            .insert((
                // physics
                Collider::capsule(1.0, 0.5),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                PLAYER_LAYER,
            ))
            .insert((
                // Inventory/store
                Inventory::default(),
                StoreHistory::default(),
                StoreBuffer::default(),
            ))
            .insert({
                let mut attrs = Attributes::default();
                attrs
                    .set(Stat::Health, 33.0)
                    .set_base(Stat::Speed, 16.0)
                    .set(Stat::CharacterResource, 33.0)
                    .set(Stat::Gold, 20_000.0);
                attrs
            })
            .insert((
                TEAM_1,
                AbilityCastSettings::default(),
                AbilityRanks::default(),
                IncomingDamageLog::default(),
                OutgoingDamageLog::default(),
                Bounty::default(),
                CooldownMap::default(),
                CCMap::default(),
                BuffMap::default(),
                Spectatable,
                Casting(None),
                WindupTimer(Timer::default()),
                PlayerInput::default(),
                SlotBundle::new(), // Has all the keybinding -> action logic
                HoveredAbility::default(),
            ))
            //.insert(NavMeshAffector)
            .id();

        let player_is_owned = event.actor == ActorType::Player(*local_player); // make it check if you are that player
        if player_is_owned {
            spectate_events.send(SpectateEvent {
                entity: player_entity,
            });
            commands.insert_resource(PlayerEntity(Some(player_entity)));
            commands.insert_resource(Spectating(player_entity));
            commands.insert_resource(PlayerInput::default());
            commands.entity(player_entity).insert((Trackable,));
        }
    }
}

fn respawn_entity(
    mut respawn_events: EventReader<RespawnEvent>,
    mut the_damned: Query<(&mut Visibility, &mut ActorState)>,
    local_player: Res<Player>,
    mut spectate_events: EventWriter<SpectateEvent>,
) {
    for event in respawn_events.read() {
        let Ok((mut vis, mut state)) = the_damned.get_mut(event.entity) else {
            continue;
        };
        *vis = Visibility::Visible;
        *state = ActorState::Alive;
        if event.actor == ActorType::Player(*local_player) {
            spectate_events.send(SpectateEvent {
                entity: event.entity,
            });
        }
    }
}

fn setup_player(mut spawn_events: EventWriter<InitSpawnEvent>, local_player: Res<Player>) {
    dbg!();
    spawn_events.send(InitSpawnEvent {
        actor: ActorType::Player(local_player.clone()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
    });
}

pub fn player_swivel(mut players: Query<(&mut Transform, &PlayerInput, &CCMap), With<Player>>) {
    for (mut player_transform, inputs, cc_map) in players.iter_mut() {
        if cc_map.map.contains_key(&CCType::Stun) {
            continue;
        }
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}

pub fn player_movement(
    mut query: Query<(
        &Attributes,
        //&mut ControllerInput,
        //&mut Movement,
        &PlayerInput,
        &CCMap,
    )>,
) {
    for (attributes, /*mut controller, mut movement,*/ player_input, cc_map) in query.iter_mut() {
        if cc_map.map.contains_key(&CCType::Root) || cc_map.map.contains_key(&CCType::Stun) {
            //controller.movement = Vec3::ZERO;
            continue;
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
        let movement_vector =
            Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32) * direction_normalized * speed;

        //controller.movement = movement_vector;
        //movement.max_speed = speed;
    }
}

pub fn cast_ability(
    mut players: Query<(&CooldownMap, &CCMap, &mut HoveredAbility)>,
    mut attempt_cast_event: EventReader<InputCastEvent>,
    mut cast_event: EventWriter<CastEvent>,
) {
    for event in attempt_cast_event.read() {
        let Ok((cooldowns, ccmap, mut hovered)) = players.get_mut(event.caster) else {
            continue;
        };
        if ccmap.map.contains_key(&CCType::Silence) || ccmap.map.contains_key(&CCType::Stun) {
            continue;
        } // play erro sound for silenced
        if cooldowns.map.contains_key(&event.ability) {
            continue;
        } // play error sound for on CD
        hovered.0 = None;
        cast_event.send(CastEvent {
            caster: event.caster,
            ability: event.ability,
        });
    }
}

#[derive(Event)]
pub struct InputCastEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Event)]
pub struct CastEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Component)]
pub struct WindupTimer(pub Timer);
pub enum CastingStage {
    Charging(Timer),
    Windup(Timer),
    None,
}

fn start_ability_windup(
    mut players: Query<(&mut WindupTimer, &mut Casting)>,
    mut cast_events: EventReader<CastEvent>,
) {
    for event in cast_events.read() {
        let Ok((mut winduptimer, mut casting)) = players.get_mut(event.caster) else {
            continue;
        };
        let windup = event.ability.get_actor_times();
        winduptimer.0 = Timer::new(
            Duration::from_millis((windup * 1000.) as u64),
            TimerMode::Once,
        );
        casting.0 = Some(event.ability);
    }
}

fn tick_windup_timer(
    time: Res<Time>,
    mut players: Query<(Entity, &mut WindupTimer, &mut Casting)>,
    mut fire_event: EventWriter<AbilityFireEvent>,
) {
    for (entity, mut timer, mut casting) in players.iter_mut() {
        let Some(ability) = casting.0 else { continue };
        timer.0.tick(time.delta());
        if timer.0.finished() {
            fire_event.send(AbilityFireEvent {
                caster: entity,
                ability: ability.clone(),
            });
            casting.0 = None;
        }
    }
}

fn trigger_cooldown(
    mut cast_events: EventReader<AbilityFireEvent>,
    mut query: Query<(&mut CooldownMap, &Attributes)>,
) {
    for event in cast_events.read() {
        let Ok((mut cooldowns, attributes)) = query.get_mut(event.caster) else {
            continue;
        };
        let cdr = 1.0 - (attributes.get(Stat::CooldownReduction) / 100.0);

        cooldowns.map.insert(
            event.ability.clone(),
            Timer::new(
                Duration::from_millis((event.ability.get_cooldown() * cdr * 1000.) as u64),
                TimerMode::Once,
            ),
        );
    }
}

// Move these to character file, since mobs will be cc'd and buffed/cooldowns
// too AND MAKE GENERIC ⬇️⬇️⬇️

fn tick_cooldowns(
    time: Res<Time>,
    mut query: Query<&mut CooldownMap>,
    //mut cd_events: EventWriter<CooldownFreeEvent>,
) {
    for mut cooldowns in &mut query {
        // remove if finished
        cooldowns.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct AbilityMap {
    pub ranks: HashMap<Ability, u32>,
    pub cds: HashMap<Ability, Timer>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct AbilityRanks {
    pub map: HashMap<Ability, Rank>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CooldownMap {
    pub map: HashMap<Ability, Timer>,
}

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

pub struct DamageSum {
    total_change: i32,
    total_mitigated: u32,
    hit_amount: u32,
    sub_list: Vec<HealthMitigatedEvent>,
}

impl DamageSum {
    pub fn add_damage(&mut self, instance: HealthMitigatedEvent) {
        self.total_change += instance.change;
        self.total_mitigated += instance.mitigated;
        self.hit_amount += 1;
        self.sub_list.push(instance);
    }

    pub fn from_instance(instance: HealthMitigatedEvent) -> Self {
        DamageSum {
            total_change: instance.change,
            total_mitigated: instance.mitigated,
            hit_amount: 1,
            sub_list: vec![instance.clone()],
        }
    }

    pub fn total_change(&self) -> i32 {
        self.total_change
    }
    pub fn total_mitigated(&self) -> u32 {
        self.total_mitigated
    }
    pub fn hit_amount(&self) -> u32 {
        self.hit_amount
    }
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
                let Some(hits) = defender_log.sums.get_mut(&damage_instance.sensor) else {
                    continue;
                };
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
                let Some(targets_hit) = attacker_log.sums.get_mut(&damage_instance.sensor) else {
                    continue;
                };
                if targets_hit.contains_key(&damage_instance.defender) {
                    let Some(hits) = targets_hit.get_mut(&damage_instance.defender) else {
                        continue;
                    };
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
                attacker_log
                    .sums
                    .insert(damage_instance.sensor.clone(), init);
                log_hit_events.send(LogHit::new(
                    damage_instance.clone(),
                    LogType::Add,
                    LogSide::Outgoing,
                ));
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogSide {
    Incoming,
    Outgoing,
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogType {
    Add,
    Stack,
}

// Change attacker to caster?
#[derive(Event, Debug)]
pub struct LogHit {
    pub sensor: Entity,
    pub attacker: Entity,
    pub defender: Entity,
    pub damage_type: DamageType,
    pub ability: Ability,
    pub change: i32,
    pub mitigated: u32,
    pub log_type: LogType,
    pub log_direction: LogSide,
}

impl LogHit {
    fn new(event: HealthMitigatedEvent, log_type: LogType, log_direction: LogSide) -> Self {
        LogHit {
            sensor: event.sensor,
            attacker: event.attacker,
            defender: event.defender,
            damage_type: event.damage_type,
            ability: event.ability,
            change: event.change,
            mitigated: event.mitigated,
            log_type,
            log_direction,
        }
    }
}

#[derive(Component)]
pub struct Tower;
