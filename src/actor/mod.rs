
use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::{
    GameState, 
    ability::{Ability}, 
    area::HealthChangeEvent, 
    game_manager::{AbilityFireEvent, TEAM_1, Bounty, CharacterState, PLAYER_GROUPING}, 
    crowd_control::{CCType, CCMap}, 
    stats::{Stat, Attributes}, 
    input::SlotBundle, ui::Trackable, 
    actor::view::{SpectateEvent, Spectatable, Spectating}, 
    buff::BuffMap};

use self::player::*;

pub mod view;
pub mod player;

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        //Resources
        app.insert_resource(PlayerInput::default());
        app.register_type::<PlayerInput>();
        app.register_type::<CooldownMap>();
        app.register_type::<HoveredAbility>();
        app.add_event::<InputCastEvent>();
        app.add_event::<CastEvent>();
        app.add_event::<SpawnEvent>();

        //Plugins

        //Systems
        app.add_system(setup_player.in_schedule(OnEnter(GameState::InGame)));
        app.add_systems((
                player_keys_input.run_if(in_state(GameState::InGame)),
                player_mouse_input.run_if(in_state(GameState::InGame)),
                select_ability.run_if(in_state(GameState::InGame)),
                update_local_player_inputs,
            )
                .in_base_set(CoreSet::PreUpdate),
        );
        app.add_systems((
                cast_ability,
                normal_casting,
                show_targetter.after(normal_casting),
                trigger_cooldown.after(cast_ability),
                tick_cooldowns.after(trigger_cooldown),
                start_ability_windup.after(cast_ability),
                tick_windup_timer,
                spawn_player,
                update_damage_logs,
            )
                .in_set(OnUpdate(GameState::InGame)),
        );
        // Process transforms always after inputs
        app.add_systems((
                actor_swivel.run_if(in_state(GameState::InGame)),
                // Process translations after rotations
                actor_movement
                    .after(actor_swivel)
                    .run_if(in_state(GameState::InGame))
            )
            .in_base_set(CoreSet::PostUpdate),
        );
    }
}

pub struct SpawnEvent {
    pub player: Player,
    pub transform: Transform,
}

#[derive(Component, Default)]
pub struct OutgoingDamageLog {
    pub list: Vec<HealthChangeEvent>,
}

#[derive(Component, Default)]
pub struct IncomingDamageLog {
    pub list: Vec<HealthChangeEvent>,
    pub ui_entities: HashMap<Entity, HealthChangeEvent>,
}


fn spawn_player(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<SpawnEvent>,
    mut spectate_events: EventWriter<SpectateEvent>,
    mut next_state: ResMut<NextState<CharacterState>>,
    local_player: Res<Player>,
) {
    for event in spawn_events.iter() {
        next_state.set(CharacterState::Alive);
        // reset the rotation so you dont spawn looking the other way

        let mut material = StandardMaterial::default();
        material.base_color = Color::hex("208000").unwrap().into();
        material.perceptual_roughness = 0.97;
        material.reflectance = 0.0;
        let green = materials.add(material);

        let spawning_id = event.player.id.clone();
        info!("spawning player {}", spawning_id);
        let player_entity = commands
            .spawn((
                SpatialBundle::from_transform(event.transform.clone()),
                _meshes.add(
                    shape::Capsule {
                        radius: 0.4,
                        ..default()
                    }
                    .into(),
                ),
                green.clone(),
                Player { id: spawning_id },
                Name::new(format!("Player {}", spawning_id.to_string())),
                Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
                ActiveEvents::COLLISION_EVENTS,
                RigidBody::Dynamic,
                Friction::coefficient(2.0),
                LockedAxes::ROTATION_LOCKED,
                Velocity::default(),
                PLAYER_GROUPING,
            )).insert({
                let mut attributes = Attributes::default();
                *attributes.entry(Stat::Health.into()).or_default() = 33.0;
                *attributes.entry(Stat::Speed.into()).or_default() = 6.0;
                *attributes.entry(Stat::CharacterResource.into()).or_default() = 33.0;
                attributes
            }).insert((
                TEAM_1,
                AbilityCastSettings::default(),
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
            .id();

        let player_is_owned = local_player.id == spawning_id; // make it check if you are that player
        if player_is_owned {
            spectate_events.send(SpectateEvent {
                entity: player_entity,
            });
            commands.insert_resource(Spectating(player_entity));
            commands.insert_resource(PlayerInput::default());
            commands.entity(player_entity).insert((
                Trackable,
            ));
        }
    }
}

fn setup_player(mut spawn_events: EventWriter<SpawnEvent>, local_player: Res<Player>) {
    spawn_events.send(SpawnEvent {
        player: local_player.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.5, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..default()
        },
    });
}

pub fn actor_swivel(mut players: Query<(&mut Transform, &PlayerInput, &CCMap), With<Player>>) {
    for (mut player_transform, inputs, cc_map) in players.iter_mut() {
        if cc_map.map.contains_key(&CCType::Stun){ continue }
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}

pub fn actor_movement(mut query: Query<(&Attributes, &mut Velocity, &PlayerInput, &CCMap)>) {
    for (attributes, mut velocity, player_input, cc_map) in query.iter_mut() {
        if cc_map.map.contains_key(&CCType::Root) || cc_map.map.contains_key(&CCType::Stun) { continue }
        let speed = *attributes.get(&Stat::Speed.into()).unwrap_or(&1.0);
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

        // don't effect the y direction since you can't move in that direction.
        velocity.linvel.x = movement_vector.x;
        velocity.linvel.z = movement_vector.z;
    }
}


pub fn cast_ability(
    mut players: Query<(&CooldownMap, &CCMap, &mut HoveredAbility)>,
    mut attempt_cast_event: EventReader<InputCastEvent>,
    mut cast_event: EventWriter<CastEvent>,
){
    for event in attempt_cast_event.iter(){
        let Ok((cooldowns, ccmap, mut hovered)) = players.get_mut(event.caster) else {continue};
        if ccmap.map.contains_key(&CCType::Silence) || ccmap.map.contains_key(&CCType::Stun) { continue } // play erro sound for silenced
        if cooldowns.map.contains_key(&event.ability) { continue } // play error sound for on CD
        hovered.0 = None;
        cast_event.send(CastEvent {
            caster: event.caster,
            ability: event.ability,
        });
    }
}

pub struct InputCastEvent {
    pub caster: Entity,
    pub ability: Ability,
}

pub struct CastEvent {
    pub caster: Entity,
    pub ability: Ability,
}


#[derive(Component)]
pub struct WindupTimer(pub Timer);
pub enum CastingStage{
    Charging(Timer),
    Windup(Timer),
    None,
}

fn start_ability_windup(
    mut players: Query<(&mut WindupTimer, &mut Casting)>,
    mut cast_events: EventReader<CastEvent>,
){
    for event in cast_events.iter(){
        let Ok((mut winduptimer, mut casting)) = players.get_mut(event.caster) else { continue };
        let windup = event.ability.get_windup();
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
){
    for (entity, mut timer, mut casting) in players.iter_mut(){
        let Some(ability) = casting.0 else {continue};
        timer.0.tick(time.delta());
        if timer.0.finished(){            
            fire_event.send(AbilityFireEvent {
                caster: entity,
                ability: ability.clone(),
            });
            casting.0 = None;
        }
    }
}

fn trigger_cooldown(mut cast_events: EventReader<AbilityFireEvent>, mut query: Query<&mut CooldownMap>) {
    for event in cast_events.iter() {
        let Ok(mut cooldowns) = query.get_mut(event.caster) else { continue };
        cooldowns.map.insert(
            event.ability.clone(),
            Timer::new(
                Duration::from_millis((event.ability.get_cooldown() * 1000.) as u64),
                TimerMode::Once,
            ),
        );
    }
}

// Move these to character file, since mobs will be cc'd and buffed/cooldowns too AND MAKE GENERIC
// ⬇️⬇️⬇️

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
pub struct CooldownMap {
    pub map: HashMap<Ability, Timer>,
}


fn update_damage_logs(
    mut incoming_logs: Query<&mut IncomingDamageLog>,
    mut outgoing_logs: Query<&mut OutgoingDamageLog>,
    mut damage_events: EventReader<HealthChangeEvent>,
){
    for damage_instance in damage_events.iter(){
        if let Some(attacker) = damage_instance.attacker{
            if let Ok(mut attacker_log) = outgoing_logs.get_mut(attacker) {
                attacker_log.list.push(damage_instance.clone());
            }
        }
        if let Ok(mut defender_log) = incoming_logs.get_mut(damage_instance.defender) {
            defender_log.list.push(damage_instance.clone());            
        }
    }
}