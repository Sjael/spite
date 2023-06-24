use std::{fmt::Debug, f32::consts::PI, collections::{HashMap}, time::{Duration}};
use bevy::{prelude::*, input::mouse::MouseMotion,};
use leafwing_input_manager::prelude::ActionState;
use serde::{Serialize, Deserialize};
use bevy_rapier3d::prelude::*;

use crate::{
    ui::{mouse::MouseState, Trackable}, 
    ability::{
        Ability, HealthChangeEvent,
    }, 
    input::{setup_player_slots, SlotAbilityMap}, 
    stats::*, 
    crowd_control::CCType, 
    game_manager::{Bounty, CharacterState, PLAYER_GROUPING, TEAM_1, CastEvent}, 
    GameState, view::{PossessEvent, Spectatable, camera_swivel_and_tilt}, buff::{BuffInfoApplied, BuffMap}};

#[derive(Component, Resource, Reflect, FromReflect, Clone, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[reflect(Component)]
pub struct Player {
    pub id: u32,
}

impl Player{
    pub fn new(id: u32) -> Self{
        Self {
            id
        }
    }
}



#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub from_height: f32,
}

#[derive(Component, Debug, Default)]
pub struct HoveredAbility(pub Ability);


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
        app.register_type::<CCMap>();
        app.register_type::<BuffMap>();
        app.add_event::<SpawnEvent>();       
        
        //Plugins

        //Systems
        app.add_system(setup_player.in_schedule(OnEnter(GameState::InGame)));
        app.add_systems((
            player_keys_input.run_if(in_state(GameState::InGame)),
            player_mouse_input.run_if(in_state(GameState::InGame)),
            select_ability.run_if(in_state(GameState::InGame)),
        ).in_base_set(CoreSet::PreUpdate));
        app.add_systems((
            cast_ability,
            trigger_cooldown.after(cast_ability),
            tick_cooldowns.after(trigger_cooldown),
            tick_ccs,
            tick_buffs,
            spawn_player,
        ).in_set(OnUpdate(GameState::InGame)));
        // Process transforms always after inputs
        app.add_systems((
            player_swivel,
            // Process translations after rotations 
            player_movement.after(player_swivel).run_if(in_state(GameState::InGame)),
            reticle_move.after(camera_swivel_and_tilt).run_if(in_state(GameState::InGame)),
            update_local_player_inputs,
        ).in_base_set(CoreSet::PostUpdate));
    }
}

pub struct SpawnEvent{
    pub player: Player,
    pub transform: Transform,
}

#[derive(Component, Default)]
pub struct OutgoingDamageLog{
    pub map: Vec<HealthChangeEvent>,
}

#[derive(Component, Default)]
pub struct IncomingDamageLog{
    pub map: Vec<HealthChangeEvent>,
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
){
    for event in spawn_events.iter(){
        next_state.set(CharacterState::Alive);
        // reset the rotation so you dont spawn looking the other way
        commands.insert_resource(PlayerInput::default());

        
        let mut material = StandardMaterial::default();
        material.base_color = Color::hex("208000").unwrap().into();
        material.perceptual_roughness = 0.97;
        material.reflectance = 0.0;
        let green = materials.add(material);

        let id  = event.player.id.clone();
        info!("spawning player {}", id);
        let player_entity = commands.spawn((
            SpatialBundle::from_transform(event.transform.clone()),
            _meshes.add(shape::Capsule{
                radius: 0.4,
                ..default()
            }.into()),       
            green.clone(), 
            Player { id },
            Name::new(format!("Player {}", id.to_string())),
            
            setup_player_slots(), // Has all the keybinding -> action logic
            PlayerInput::default(),
        
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
            ActiveEvents::COLLISION_EVENTS,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            PLAYER_GROUPING,
        ))
        .insert((
            Attribute::<Health>::new(33.0),
            Attribute::<Min<Health>>::new(0.0),
            Attribute::<Max<Health>>::new(235.0),
            Attribute::<Regen<Health>>::new(9.5),
        ))
        .insert((
            Attribute::<CharacterResource>::new(175.0),
            Attribute::<Min<CharacterResource>>::new(0.0),
            Attribute::<Max<CharacterResource>>::new(400.0),
            Attribute::<Regen<CharacterResource>>::new(2.0),
        ))
        .insert((
            Attribute::<MovementSpeed>::new(2.0),
            Attribute::<Base<MovementSpeed>>::new(5.0),
            Attribute::<Plus<MovementSpeed>>::new(1.0),
            Attribute::<Mult<MovementSpeed>>::new(1.0),
        ))
        .insert((
            TEAM_1,
            HoveredAbility::default(),
            IncomingDamageLog::default(),
            OutgoingDamageLog::default(),
            Bounty::default(),
            CooldownMap::default(),
            CCMap::default(),
            BuffMap::default(),
            NetworkOwner,
            Spectatable,
            Trackable,
        ))
        .id();

        let player_is_owned = true; // make it check if you are that player
        if player_is_owned {
            possess_events.send(PossessEvent{
                entity: player_entity,
            })
        }
        
    }
   
}


fn setup_player(
    mut spawn_events: EventWriter<SpawnEvent>,
){    
    spawn_events.send(SpawnEvent{
        player: Player{ id: 1507 },
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
    mouse_state: Res<State<MouseState>>,
) {
    if mouse_state.0 == MouseState::Free{ // if mouse is free, dont turn character
        return;
    }
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.iter() {
        cumulative_delta += ev.delta;
    }
    // turn into resource later
    let sens = 1.0;
    player_input.pitch -= sens * cumulative_delta.y / 180.0;
    player_input.pitch = player_input.pitch.clamp(-PI / 2.0, PI / 2.0);
    player_input.yaw -= sens * cumulative_delta.x / 180.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);    
}

fn player_keys_input(
    keyboard_input: Res<Input<KeyCode>>, 
    mut player_input: ResMut<PlayerInput>,
) {    
    // only 1 player right now
    player_input.set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
    player_input.set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
    player_input.set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
    player_input.set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));
    player_input.set_ability1(keyboard_input.pressed(KeyCode::Key1));
    player_input.set_ability2(keyboard_input.pressed(KeyCode::Key2));
    player_input.set_ability3(keyboard_input.pressed(KeyCode::Key3));
    player_input.set_ability4(keyboard_input.pressed(KeyCode::Key4));
}

pub fn update_local_player_inputs(
    player_input: Res<PlayerInput>,
    mut query: Query<&mut PlayerInput>,
) {
    if let Ok(mut input) = query.get_single_mut() {
        //info!("setting local player inputs: {:?}", player_input);
        *input = player_input.clone();
    } else {
        //warn!("no player to provide input for");
    }
}

fn player_swivel(
    mut players: Query<(&mut Transform, &PlayerInput), With<Player>>,
){
    for (mut player_transform, inputs) in players.iter_mut() {
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}


pub fn player_movement(mut query: Query<(&Attribute<MovementSpeed>, &mut Velocity, &PlayerInput)>) {
    for (speed, mut velocity, player_input) in query.iter_mut() {
        //dbg!(player_input);
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
            Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32) * direction_normalized * *speed.amount();

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

fn select_ability(
    mut query: Query<(&mut HoveredAbility, &ActionState<Ability>)>,
) {
    // TODO get specific player
    for (mut hover, ab_state) in &mut query {
        for ability in ab_state.get_just_pressed() {
            if hover.0 != ability {
                hover.0 = ability.clone();
            }
        }
    }
}

pub fn cast_ability(
    players: Query<(&CooldownMap, &Player, Entity, &ActionState<Ability>, &Children)>,
    //reticle: Query<(&GlobalTransform, &Reticle), Without<Player>>,
    mut cast_event: EventWriter<CastEvent>,
){
    for (cooldowns, _, player_entity, ability_actions, _) in &players {
        for ability in ability_actions.get_just_pressed() {
            if !cooldowns.map.contains_key(&ability){
                cast_event.send(CastEvent {
                    caster: player_entity,
                    ability: ability.clone(),
                });
            }
        }
    }
}


fn trigger_cooldown(
    mut cast_events: EventReader<CastEvent>,
    mut query: Query<(&Player, &mut CooldownMap, &SlotAbilityMap, Entity)>,
) {
    for event in cast_events.iter() {
        
        for (_, mut cooldowns, _, _e) in &mut query {
            cooldowns.map.insert(
                event.ability.clone(),
                Timer::new(Duration::from_millis((event.ability.get_cooldown() * 1000.) as u64), TimerMode::Once),
            );
        }
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

fn tick_ccs(
    time: Res<Time>,
    mut query: Query<&mut CCMap>,
) {
    for mut ccs in &mut query {
        ccs.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}
fn tick_buffs(
    time: Res<Time>,
    mut query: Query<&mut BuffMap>,
) {
    for mut buffs in &mut query {
        buffs.map.retain(|_, buff| {
            buff.timer.tick(time.delta());
            !buff.timer.finished()
        });
    }
}


#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CooldownMap {
    pub map: HashMap<Ability, Timer>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CCMap {
    pub map: HashMap<CCType, Timer>,
}




