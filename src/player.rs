use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f32::consts::PI, fmt::Debug, time::Duration};

use crate::{
    ability::{Ability, HealthChangeEvent, shape::AbilityShape, ability_bundles::Targetter},
    buff::BuffMap,
    crowd_control::{CCType, CCMap},
    game_manager::{Bounty, CharacterState, PLAYER_GROUPING, TEAM_1, AbilityFireEvent},
    input::{SlotBundle, MouseSensitivity},
    stats::*,
    ui::{mouse::MouseState, Trackable},
    view::{camera_swivel_and_tilt, PossessEvent, Spectatable},
    GameState,
};

#[derive(Component, Resource, Reflect, FromReflect, Clone, Debug, Default, 
    PartialEq, Serialize, Deserialize, Eq, Hash, Deref, DerefMut,)]
#[reflect(Component)]
pub struct Player {
    pub id: u32,
}

impl Player {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub from_height: f32,
}

#[derive(Component, Debug, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct HoveredAbility(pub Option<Ability>);
#[derive(Component, Debug, Default)]
pub struct Casting(pub Option<Ability>);

#[derive(Debug, PartialEq, Eq)]
pub enum CastType{
    Normal,
    Quick,
    Instant,
}
#[derive(Component, Debug)]
pub struct AbilityCastSettings(pub HashMap<Ability, CastType>);

impl Default for AbilityCastSettings{
    fn default() -> Self {
        let settings = HashMap::from([
            (Ability::BasicAttack, CastType::Instant),
        ]);
        Self(settings)
    }
}

#[derive(Component, Resource, Reflect, Clone, Copy, Default, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct PlayerInput {
    /// Movement inputs
    pub binary_inputs: PlayerInputKeys,
    /// Vertical rotation of camera
    pub pitch: f32,
    /// Horizontal rotation of camera
    pub yaw: f32,
    //pub casted: [Option<CastInput>; 4],
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            binary_inputs: PlayerInputKeys::empty(),
            pitch: 0.0,
            yaw: 0.0,
            //casted: [None; 4],
        }
    }

    pub fn set_forward(&mut self, forward: bool) {
        self.binary_inputs.set(PlayerInputKeys::FORWARD, forward);
    }
    pub fn set_back(&mut self, back: bool) {
        self.binary_inputs.set(PlayerInputKeys::BACK, back);
    }
    pub fn set_left(&mut self, left: bool) {
        self.binary_inputs.set(PlayerInputKeys::LEFT, left);
    }
    pub fn set_right(&mut self, right: bool) {
        self.binary_inputs.set(PlayerInputKeys::RIGHT, right);
    }
    pub fn forward(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::FORWARD)
    }
    pub fn back(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::BACK)
    }
    pub fn left(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::LEFT)
    }
    pub fn right(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::RIGHT)
    }
    pub fn set_ability1(&mut self, pressed: bool) {
        self.binary_inputs.set(PlayerInputKeys::ABILITY_1, pressed);
    }
    pub fn set_ability2(&mut self, pressed: bool) {
        self.binary_inputs.set(PlayerInputKeys::ABILITY_2, pressed);
    }
    pub fn set_ability3(&mut self, pressed: bool) {
        self.binary_inputs.set(PlayerInputKeys::ABILITY_3, pressed);
    }
    pub fn set_ability4(&mut self, pressed: bool) {
        self.binary_inputs.set(PlayerInputKeys::ABILITY_4, pressed);
    }
    pub fn ability1(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::ABILITY_1)
    }
    pub fn ability2(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::ABILITY_2)
    }
    pub fn ability3(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::ABILITY_3)
    }
    pub fn ability4(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::ABILITY_4)
    }
    pub fn set_left_click(&mut self, clicked: bool) {
        self.binary_inputs.set(PlayerInputKeys::LEFT_CLICK, clicked);
    }
    pub fn set_right_click(&mut self, clicked: bool) {
        self.binary_inputs.set(PlayerInputKeys::RIGHT_CLICK, clicked);
    }
    pub fn left_click(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::LEFT_CLICK)
    }
    pub fn right_click(&self) -> bool {
        self.binary_inputs.contains(PlayerInputKeys::RIGHT_CLICK)
    }
}

bitflags::bitflags! {
    #[derive(Default, Serialize, Deserialize, Reflect)]
    pub struct PlayerInputKeys: u16 {
        const FORWARD = 1 << 1;
        const BACK = 1 << 2;
        const LEFT = 1 << 3;
        const RIGHT = 1 << 4;

        const ABILITY_1 = 1 << 5;
        const ABILITY_2 = 1 << 6;
        const ABILITY_3 = 1 << 7;
        const ABILITY_4 = 1 << 8;

        const LEFT_CLICK = 1 << 9;
        const RIGHT_CLICK = 1 << 10;
    }
}

impl PlayerInputKeys {
    pub fn shorthand_display(self) -> String {
        let mut keys = "".to_owned();

        keys += match self.contains(Self::LEFT) {
            true => "<",
            false => "-",
        };

        keys += match self.contains(Self::FORWARD) {
            true => "^",
            false => "-",
        };

        keys += match self.contains(Self::BACK) {
            true => "v",
            false => "-",
        };

        keys += match self.contains(Self::RIGHT) {
            true => ">",
            false => "-",
        };
        keys += match self.contains(Self::LEFT_CLICK) {
            true => "L",
            false => "-",
        };
        keys += match self.contains(Self::RIGHT_CLICK) {
            true => "R",
            false => "-",
        };

        if self.contains(Self::ABILITY_1) {
            keys += "1";
        };
        if self.contains(Self::ABILITY_2) {
            keys += "2";
        };
        if self.contains(Self::ABILITY_3) {
            keys += "3";
        };
        if self.contains(Self::ABILITY_4) {
            keys += "4";
        };

        keys
    }
}

impl Debug for PlayerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerInput")
            .field("keys", &self.binary_inputs.shorthand_display())
            .field("pitch", &Radians(self.pitch))
            .field("yaw", &Radians(self.yaw))
            .finish()
    }
}

#[derive(Clone, Copy, Default, Component, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Radians(f32);

impl Debug for Radians {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{value:.precision$?}°",
            precision = f.precision().unwrap_or(2),
            value = self.0 * 360.0 / std::f32::consts::TAU,
        ))
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
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
                player_swivel,
                // Process translations after rotations
                player_movement
                    .after(player_swivel)
                    .run_if(in_state(GameState::InGame)),
                reticle_move
                    .after(camera_swivel_and_tilt)
                    .run_if(in_state(GameState::InGame)),
                update_local_player_inputs,
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

#[derive(Component)]
pub struct NetworkOwner;

fn spawn_player(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<SpawnEvent>,
    mut possess_events: EventWriter<PossessEvent>,
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
            ))
            .id();

        let player_is_owned = local_player.id == spawning_id; // make it check if you are that player
        if player_is_owned {
            possess_events.send(PossessEvent {
                entity: player_entity,
            });
            commands.insert_resource(PlayerInput::default());
            commands.entity(player_entity).insert((
                Trackable,
                NetworkOwner,
                PlayerInput::default(),
                SlotBundle::new(), // Has all the keybinding -> action logic
                HoveredAbility::default(),
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

fn player_mouse_input(
    mut ev_mouse: EventReader<MouseMotion>,
    mut player_input: ResMut<PlayerInput>,
    sensitivity: Res<MouseSensitivity>,
    mouse_state: Res<State<MouseState>>,
) {
    if mouse_state.0 == MouseState::Free { return; } // if mouse is free, dont turn character
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.iter() {
        cumulative_delta += ev.delta;
    }
    player_input.pitch -= sensitivity.0 * cumulative_delta.y / 180.0;
    player_input.pitch = player_input.pitch.clamp(-PI / 2.0, PI / 2.0);
    player_input.yaw -= sensitivity.0 * cumulative_delta.x / 180.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);
}

// change to use leafwing slots? 
fn player_keys_input(
    keyboard_input: Res<Input<KeyCode>>, 
    mouse_input: Res<Input<MouseButton>>, 
    mut player_input: ResMut<PlayerInput>
) {
    player_input.set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
    player_input.set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
    player_input.set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
    player_input.set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));
    player_input.set_ability1(keyboard_input.pressed(KeyCode::Key1));
    player_input.set_ability2(keyboard_input.pressed(KeyCode::Key2));
    player_input.set_ability3(keyboard_input.pressed(KeyCode::Key3));
    player_input.set_ability4(keyboard_input.pressed(KeyCode::Key4));
    player_input.set_left_click(mouse_input.pressed(MouseButton::Left));
    player_input.set_right_click(mouse_input.pressed(MouseButton::Right));
}

pub fn update_local_player_inputs(
    player_input: Res<PlayerInput>,
    mut query: Query<&mut PlayerInput>,
) {
    let Ok(mut input) = query.get_single_mut() else { return };
    *input = player_input.clone();
    //info!("setting local player inputs: {:?}", player_input);
}

fn player_swivel(mut players: Query<(&mut Transform, &PlayerInput), With<Player>>) {
    for (mut player_transform, inputs) in players.iter_mut() {
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}

pub fn player_movement(mut query: Query<(&Attributes, &mut Velocity, &PlayerInput)>) {
    for (attributes, mut velocity, player_input) in query.iter_mut() {
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

fn reticle_move(
    players: Query<(&PlayerInput, Option<&Children>), With<Player>>,
    mut reticles: Query<(&mut Transform, &Reticle), Without<Player>>,
) {
    for (player_input, children) in players.iter() {
        let Some(children) = children else { continue };
        for child in children.iter() {
            let Ok((mut transform, reticle)) = reticles.get_mut(*child) else { continue };
            let current_angle = player_input.pitch.clamp(-1.57, 0.);
            // new poggers way
            transform.translation.z = (1.57 + current_angle).tan() * -reticle.from_height;
            transform.translation.z = transform.translation.z.clamp(-reticle.max_distance, 0.);
        }
    }
}


fn show_targetter(
    mut commands: Commands,
    query: Query<(&HoveredAbility, Entity, &Children), Changed<HoveredAbility>>,
    reticles: Query<Entity, With<Reticle>>,
    targetters: Query<(Entity, &Ability), With<Targetter>>,
    children_query: Query<&Children>,
){
    for (hovered, entity, children) in &query{
        let mut already_has_targetter = false;
        for descendant in children_query.iter_descendants(entity){
            let Ok((targetter_entity, old_ability)) = targetters.get(descendant) else {continue};
            if let Some(hovered_ability) = hovered.0{
                if hovered_ability != *old_ability{
                    commands.entity(targetter_entity).despawn_recursive();
                } else {
                    already_has_targetter = true;
                }
            } else{
                commands.entity(targetter_entity).despawn_recursive();
            }
        }

        let Some(hovered_ability) = hovered.0 else { continue };

        let mut reticle = None;
        for child in children.iter(){
            if let Ok(found_reticle) = reticles.get(*child) {
                reticle = Some(found_reticle);
            };
        }

        if !already_has_targetter{
            let targetter = commands.spawn((
                SpatialBundle::from_transform(Transform {
                    //translation: Vec3::new(10.0, 0.0, -10.0),
                    ..default()
                }),
                AbilityShape::Arc {
                    radius: 1.,
                    angle: 360.,
                },
                Sensor,
                Targetter,
                hovered_ability.clone(),
            )).id();
    
            commands.entity(targetter).set_parent(entity);
        }
    }
}

// Make this local only? would be weird to sync other players cast settings, but sure?
fn select_ability(
    mut query: Query<(&mut HoveredAbility, &ActionState<Ability>, &AbilityCastSettings, Entity)>,
    mut cast_event: EventWriter<InputCastEvent>,
) {
    for (mut hover, ab_state, cast_settings, caster_entity) in &mut query {
        for ability in ab_state.get_just_pressed() {
            let cast_type = cast_settings.0.get(&ability).unwrap_or(&CastType::Normal);
            if *cast_type == CastType::Normal{
                hover.0 = Some(ability.clone());
            } else if *cast_type == CastType::Instant{
                cast_event.send(InputCastEvent {
                    caster: caster_entity,
                    ability: ability,
                });
            }
        }
    }
}

fn normal_casting(
    mut query: Query<(&PlayerInput, &mut HoveredAbility, Entity)>,
    mut cast_event: EventWriter<InputCastEvent>,
    mouse: Res<Input<MouseButton>>, // change to track input better later, have .just_pressed() on PlayerInput
){
    for (_input, mut hovered, player) in &mut query{
        let Some(hovered_ability) = hovered.0 else {continue};
        if mouse.just_pressed(MouseButton::Left){
            cast_event.send(InputCastEvent {
                caster: player,
                ability: hovered_ability,
            });
        }
        if mouse.just_pressed(MouseButton::Right){
            hovered.0 = None;
        }
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