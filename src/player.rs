use std::{fmt::Debug, f32::consts::PI, collections::{HashMap, BTreeMap}, time::{Duration, Instant}};
use bevy::{prelude::*, input::mouse::MouseMotion, core_pipeline::{tonemapping::{Tonemapping, DebandDither}, fxaa::Fxaa, bloom::BloomSettings}};
use leafwing_input_manager::prelude::ActionState;
use serde::{Serialize, Deserialize};
use bevy_rapier3d::prelude::*;

use crate::{
    ui::mouse::MouseState, 
    ability::{
        Ability,
        ability_bundles::{FrostboltInfo, DefaultAbilityInfo, FireballInfo}, DamageInstance,
    }, 
    input::{setup_player_slots, SlotAbilityMap}, 
    stats::*, 
    crowd_control::CCType, 
    game_manager::{Bounty, CharacterState, Team}, 
    GameState};

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
pub struct Neck;

#[derive(Component, Debug)]
pub struct PlayerCam;

#[derive(Component, Clone, Debug)]
pub struct AvoidIntersecting {
    pub dir: Vec3,
    pub max_toi: f32,
    pub buffer: f32,
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
        app.add_event::<CastEvent>();
        app.add_event::<SpawnEvent>();        
        
        //Plugins

        //Systems
        app.add_system(setup_player.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(
            avoid_intersecting.in_schedule(CoreSchedule::FixedUpdate).in_set(OnUpdate(GameState::InGame))
        );
        app.add_systems((
            player_keys_input.run_if(in_state(GameState::InGame)),
            player_mouse_input.run_if(in_state(GameState::InGame)),
            select_ability.run_if(in_state(GameState::InGame)),
        ).in_base_set(CoreSet::PreUpdate));
        app.add_systems((
            cast_ability,
            place_ability.after(cast_ability),
            trigger_cooldown.after(cast_ability),
            tick_cooldowns.after(trigger_cooldown),
            spawn_player,
            tick_ccs,
            tick_buffs,
        ).in_set(OnUpdate(GameState::InGame)));
        // Process transforms always after inputs
        app.add_systems((
            player_swivel_and_tilt,
            // Process translations after rotations 
            player_movement.after(player_swivel_and_tilt).run_if(in_state(GameState::InGame)),
            reticle_move.after(player_swivel_and_tilt).run_if(in_state(GameState::InGame)),
        ).in_base_set(CoreSet::PostUpdate));
    }
}

pub struct SpawnEvent{
    pub player: Player,
    pub transform: Transform,
}
pub struct PossessEvent{
    pub entity_to_possess: Option<Entity>,
    pub player: Player,
}

#[derive(Component, Default)]
pub struct OutgoingDamageLog{
    pub map: Vec<DamageInstance>,
}

#[derive(Component, Default)]
pub struct IncomingDamageLog{
    pub map: Vec<DamageInstance>,
    pub ui_entities: HashMap<Entity, DamageInstance>,
}


fn spawn_player(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<SpawnEvent>,
    mut next_state: ResMut<NextState<CharacterState>>,
){
    for event in spawn_events.iter(){
        next_state.set(CharacterState::Alive);
        
        let mut material = StandardMaterial::default();
        material.base_color = Color::hex("800000").unwrap().into();
        material.perceptual_roughness = 0.97;
        material.reflectance = 0.0;
        let red = materials.add(material);

        let id  = event.player.id.clone();
        info!("spawning player {}", id);
        let player_entity = commands.spawn((
            SpatialBundle::from_transform(event.transform.clone()),
            _meshes.add(shape::Capsule{
                radius: 0.4,
                ..default()
            }.into()),       
            red.clone(), 
            Player { id },
            Name::new(format!("Player {}", id.to_string())),
            
            setup_player_slots(), // Has all the keybinding -> action logic
            PlayerInput::default(),
        
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
            ActiveEvents::COLLISION_EVENTS,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
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
            Attribute::<Regen<CharacterResource>>::new(15.0),
        ))
        .insert((
            Attribute::<MovementSpeed>::new(2.0),
            Attribute::<Base<MovementSpeed>>::new(5.0),
            Attribute::<Plus<MovementSpeed>>::new(1.0),
            Attribute::<Mult<MovementSpeed>>::new(1.0),
        ))
        .insert((
            Team(1),
            HoveredAbility::default(),
            IncomingDamageLog::default(),
            OutgoingDamageLog::default(),
            Bounty::default(),
            CooldownMap::default(),
            CCMap::default(),
            BuffMap::default(),
        ))
        .id();

        let camera = commands.spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 6.5))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::ReinhardLuminance,
                dither: DebandDither::Enabled,
                camera: Camera{
                    order: 1, // overwite the spectator cam
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            Fxaa::default(),
            PlayerCam,
            BloomSettings::default(),
            Name::new("Player Camera"),
            AvoidIntersecting {
                dir: Vec3::Z,
                max_toi: 6.5,
                buffer: 0.05,
            },
            /* 
            FogSettings {
                color: Color::rgba(0.05, 0.1, 0.4, 1.0),
                falloff: FogFalloff::from_visibility_colors(
                    10000.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
                ..default()
            },*/
        )).id();

        let neck = commands.spawn((
            SpatialBundle::from_transform(
                Transform {
                    translation: Vec3::new(0., 1., 0.),
                    ..default()
            }),
            Neck,
            Name::new("Neck"),
        )).id();
        
        let reticle = commands.spawn((
            SpatialBundle::from_transform(
                Transform {
                    translation: Vec3::new(0., 0., 0.),
                    ..default()
            }),
            Reticle {
                max_distance: 7.0,
                from_height: 4.0,
            },
            Name::new("Reticle"),
        )).id();      
        
        let ret_mesh = commands.spawn(PbrBundle {
            material: red.clone(),
            mesh: _meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 })),
            ..default()
        }).id();

        commands.entity(neck).push_children(&[camera]);
        commands.entity(reticle).push_children(&[ret_mesh]);
        
        commands
            .entity(player_entity)
            .push_children(&[neck, reticle]);
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


pub fn avoid_intersecting(
    rapier_context: Res<RapierContext>,
    global_query: Query<&GlobalTransform>,
    mut avoid: Query<(&mut Transform, &Parent, &AvoidIntersecting)>,
) {
    let filter = QueryFilter::exclude_dynamic().exclude_sensors();
    for (mut transform, parent, avoid) in &mut avoid {
        let adjusted_transform = if let Ok(global) = global_query.get(parent.get()) {
            global.compute_transform()
        } else {
            Transform::default()
        };
        let (toi, normal) = if let Some((_entity, intersection)) = rapier_context
            .cast_ray_and_get_normal(
                adjusted_transform.translation,
                adjusted_transform.rotation * avoid.dir,
                avoid.max_toi + avoid.buffer,
                true,
                filter,
            ) {
            (intersection.toi, intersection.normal)
        } else {
            (avoid.max_toi + avoid.buffer, Vec3::ZERO)
        };
        transform.translation = avoid.dir * toi + (normal * avoid.buffer);
    }
}

fn player_mouse_input(
    mut ev_mouse: EventReader<MouseMotion>,
    //mut player_input: ResMut<PlayerInput>,
    mut query: Query<&mut PlayerInput>,
    mouse_state: Res<State<MouseState>>,
) {
    if mouse_state.0 == MouseState::Free{ // if mouse is free, dont turn character
        return;
    }
    let Ok(mut player_input) = query.get_single_mut() else { return };
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
    mut query: Query<&mut PlayerInput>,
) {    
    // only 1 player right now
    if let Ok(mut player_input) = query.get_single_mut() {
        player_input
            .set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
        player_input
            .set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
        player_input
            .set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
        player_input
            .set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));
        player_input
            .set_ability1(keyboard_input.pressed(KeyCode::Key1));
        player_input
            .set_ability2(keyboard_input.pressed(KeyCode::Key2));
        player_input
            .set_ability3(keyboard_input.pressed(KeyCode::Key3));
        player_input
            .set_ability4(keyboard_input.pressed(KeyCode::Key4));
    }
}

fn player_swivel_and_tilt(
    mut players: Query<(&mut Transform, &PlayerInput, Option<&Children>), With<Player>>,
    mut necks: Query<&mut Transform, (With<Neck>, Without<Player>)>,
    keyboard_input: Res<Input<KeyCode>>, 
) {
    for (mut player_transform, inputs, children) in players.iter_mut() {
    
        let Some(children) = children else { return };
        for child in children.iter() {
            let Ok(mut neck_transform) = necks.get_mut(*child) else { continue };
            neck_transform.rotation = Quat::from_axis_angle(Vec3::X, inputs.pitch as f32).into();
        }
        if keyboard_input.just_released(KeyCode::LShift){
            //outer_gimbal.rotation = Quat::IDENTITY;
        } else if keyboard_input.pressed(KeyCode::LShift){
            //outer_gimbal.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
        } else{
            player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
        }
    }
}

fn player_movement(mut query: Query<(&Attribute<MovementSpeed>, &mut Velocity, &PlayerInput)>) {
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

fn cast_ability(
    players: Query<(&CooldownMap, &Player, Entity, &ActionState<Ability>, &Children)>,
    //reticle: Query<(&GlobalTransform, &Reticle), Without<Player>>,
    mut cast_event: EventWriter<CastEvent>,
){
    for (cooldowns, _, player_entity, ability_actions, _) in &players {
        for ability in ability_actions.get_just_pressed() {
            if !cooldowns.map.contains_key(&ability){
                cast_event.send(CastEvent {
                    player: player_entity,
                    ability: ability.clone(),
                });
            }
        }
    }
}

fn place_ability(
    mut commands: Commands,
    mut cast_events: EventReader<CastEvent>,
    player: Query<&GlobalTransform, (With<SlotAbilityMap>, With<Player>)>,
    reticle: Query<&GlobalTransform, (With<Reticle>, Without<Player>)>,
){
    // TODO implement player id for multiplayer, and its child reticle
    match (player.get_single(), reticle.get_single()){
        (Ok(player_transform), Ok(reticle_transform)) => {

            for event in cast_events.iter() {
                // this was doing Ability impl
                //let spawned_ability = event.ability.fire(&mut commands);
                let _spawned = match event.ability{
                    Ability::Frostbolt => {
                        let blueprint = FrostboltInfo::default();
                        blueprint.fire(&mut commands, &player_transform.compute_transform())
                    },
                    Ability::Fireball => {
                        let blueprint = FireballInfo::default();
                        blueprint.fire(&mut commands, &reticle_transform.compute_transform())
                    },
                    _ => { 
                        let blueprint = DefaultAbilityInfo::default();
                        blueprint.fire(&mut commands, &player_transform.compute_transform())
                    },
                };
            }
        }
        _ => ()
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

// Move these to character file, since mobs will be cc'd and buffed/cooldowns too
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
        buffs.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}

pub struct CastEvent {
    pub player: Entity,
    pub ability: Ability,
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

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct BuffMap {
    pub map: HashMap<String, Timer>, // Create buff id from entity-ability/item-positive, orc2-spear-debuff aka who it comes from
}



