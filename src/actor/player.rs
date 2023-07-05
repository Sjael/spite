use bevy::{input::mouse::MouseMotion, prelude::*};
use leafwing_input_manager::prelude::ActionState;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f32::consts::PI, fmt::Debug};


use crate::{ability::{Ability, bundles::Targetter}, ui::mouse::MouseState, input::MouseSensitivity, assets::MaterialPresets};

use super::{InputCastEvent, view::OuterGimbal, CooldownMap};

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


pub fn player_mouse_input(
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
pub fn player_keys_input(
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
    mut query: Query<(&mut PlayerInput, &Player)>,
    local_player: Res<Player>,
) {
    for (mut input, player) in &mut query{
        if player.id != local_player.id { continue }
        *input = player_input.clone();
        //info!("setting local player inputs: {:?}", player_input);
    }
}

pub fn show_targetter(
    mut commands: Commands,
    query: Query<(&HoveredAbility, &CooldownMap), Changed<HoveredAbility>>,
    reticles: Query<Entity, With<Reticle>>,
    gimbals: Query<Entity, With<OuterGimbal>>,
    targetters: Query<(Entity, &Ability), With<Targetter>>,
    presets: Res<MaterialPresets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    for (hovered, cooldowns) in &query{
        let Ok(reticle_entity) = reticles.get_single() else {continue};
        let Ok(gimbal_entity) = gimbals.get_single() else {continue};
        let mut already_has_targetter = false;
        for (targetter_entity, old_ability) in &targetters{
            if let Some(hovered_ability) = hovered.0{
                if hovered_ability != *old_ability{
                    commands.entity(targetter_entity).despawn_recursive();
                } else {
                    already_has_targetter = true;
                }
            } else {
                commands.entity(targetter_entity).despawn_recursive();
            }
        }
        let Some(hovered_ability) = hovered.0 else { continue };

        if !already_has_targetter{
            let mut handle = presets.0.get("blue").unwrap_or(&materials.add(Color::rgb(0.1, 0.2, 0.7).into())).clone();
            if cooldowns.map.contains_key(&hovered_ability) {
                handle = presets.0.get("white").unwrap_or(&materials.add(Color::rgb(0.4, 0.4, 0.4).into())).clone();
            }
            let targetter = hovered_ability.get_targetter(&mut commands);
            commands.entity(targetter).insert((
                hovered_ability.clone(),
                handle,
            ));
            
            if hovered_ability.on_reticle(){
                commands.entity(targetter).set_parent(reticle_entity);
            } else{
                commands.entity(targetter).set_parent(gimbal_entity);
            }
        }
    }
}

pub fn change_targetter_color(
    query: Query<(&HoveredAbility, &CooldownMap), Changed<CooldownMap>>,
    mut targetters: Query<(&Ability, &mut Handle<StandardMaterial>), With<Targetter>>,
    presets: Option<Res<MaterialPresets>>,
){  
    let Some(presets) = presets else { return } ;
    let Some(handle) = presets.0.get("blue") else {return};
    for (hovered, cooldowns) in &query{
        let Some(hovered_ability) = hovered.0 else { continue };
        if cooldowns.map.contains_key(&hovered_ability) { continue }
        for (old_ability, mut material) in &mut targetters{
            if old_ability != &hovered_ability { continue }
            *material = handle.clone();            
        }
    }
}


// Make this local only? would be weird to sync other players cast settings, but sure?
pub fn select_ability(
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

pub fn normal_casting(
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
