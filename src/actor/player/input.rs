use std::f32::consts::PI;

use bevy::{ecs::query::WorldQuery, input::mouse::MouseMotion};
use serde::{Deserialize, Serialize};

use crate::{prelude::*, ui::mouse::MouseState};

use super::{LocalPlayerId, Player};

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputCollectionSet;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerInput>();

        app.insert_resource(PlayerInput::default());
        app.insert_resource(MouseSensitivity(1.0));

        app.configure_sets(PreUpdate, InputCollectionSet.in_set(InGameSet::Pre));

        app.add_systems(
            PreUpdate,
            (
                player_mouse_input,
                player_keys_input,
                update_local_player_inputs,
            )
                .chain()
                .in_set(InputCollectionSet),
        );

        app.add_systems(
            FixedUpdate,
            (previous::<PlayerInput>,).in_set(InGameSet::Post),
        );
    }
}

/// Query information about the changing state of player inputs.
#[derive(WorldQuery)]
pub struct PlayerInputQuery {
    current: &'static PlayerInput,
    previous: &'static Previous<PlayerInput>,
}

impl<'w> PlayerInputQueryItem<'w> {
    pub fn pressed(&self, action: PlayerInputKeys) -> bool {
        self.current.pressed(action)
    }

    pub fn just_pressed(&self, action: PlayerInputKeys) -> bool {
        self.current.pressed(action) && !self.previous.pressed(action)
    }

    pub fn just_released(&self, action: PlayerInputKeys) -> bool {
        !self.current.pressed(action) && self.previous.pressed(action)
    }

    pub fn slots(&self) -> &[PlayerInputKeys] {
        self.current.slots()
    }
}

/// I don't think we should be doing this tbh
/// we should probably just use the [`LocalPlayer`] resource to fetch it from the entity.
pub fn update_local_player_inputs(
    player_input: Res<PlayerInput>,
    mut query: Query<(&mut PlayerInput, &Player)>,
    local_player: Res<LocalPlayerId>,
) {
    for (mut input, player) in &mut query {
        if *player != **local_player {
            continue;
        }
        *input = player_input.clone();
        //info!("setting local player inputs: {:?}", player_input);
    }
}

/// Collect inputs for turning the pitch and yaw of the character.
pub fn player_mouse_input(
    mut ev_mouse: EventReader<MouseMotion>,
    mut player_input: ResMut<PlayerInput>,
    sensitivity: Res<MouseSensitivity>,
    mouse_state: Res<State<MouseState>>,
) {
    if *mouse_state == MouseState::Free {
        return;
    } // if mouse is free, dont turn character
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.read() {
        cumulative_delta += ev.delta;
    }
    player_input.pitch -= sensitivity.0 * cumulative_delta.y / 180.0;
    player_input.pitch = player_input.pitch.clamp(-PI / 2.0, PI / 2.0);
    player_input.yaw -= sensitivity.0 * cumulative_delta.x / 180.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);
}

// change to use leafwing slots? Also input component?
/// Propagate local keyboard/mouse inputs into the [`PlayerInput`].
pub fn player_keys_input(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut player_input: ResMut<PlayerInput>,
) {
    player_input
        .set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
    player_input
        .set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
    player_input
        .set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
    player_input
        .set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));

    player_input.set_ability1(keyboard_input.pressed(KeyCode::Key1));
    player_input.set_ability2(keyboard_input.pressed(KeyCode::Key2));
    player_input.set_ability3(keyboard_input.pressed(KeyCode::Key3));
    player_input.set_ability4(keyboard_input.pressed(KeyCode::Key4));
    player_input.set_left_click(mouse_input.pressed(MouseButton::Left));
    player_input.set_right_click(mouse_input.pressed(MouseButton::Right));
}

/// Ground truth collection of player inputs for this simulation tick.
#[derive(Component, Resource, Reflect, Clone, Copy, Default, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct PlayerInput {
    /// Movement inputs
    pub binary_inputs: PlayerInputKeys,
    /// Vertical rotation of camera
    pub pitch: f32,
    /// Horizontal rotation of camera
    pub yaw: f32,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            binary_inputs: PlayerInputKeys::empty(),
            pitch: 0.0,
            yaw: 0.0,
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
        self.pressed(PlayerInputKeys::FORWARD)
    }
    pub fn back(&self) -> bool {
        self.pressed(PlayerInputKeys::BACK)
    }
    pub fn left(&self) -> bool {
        self.pressed(PlayerInputKeys::LEFT)
    }
    pub fn right(&self) -> bool {
        self.pressed(PlayerInputKeys::RIGHT)
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
        self.pressed(PlayerInputKeys::ABILITY_1)
    }
    pub fn ability2(&self) -> bool {
        self.pressed(PlayerInputKeys::ABILITY_2)
    }
    pub fn ability3(&self) -> bool {
        self.pressed(PlayerInputKeys::ABILITY_3)
    }
    pub fn ability4(&self) -> bool {
        self.pressed(PlayerInputKeys::ABILITY_4)
    }
    pub fn slots(&self) -> &[PlayerInputKeys] {
        &[
            PlayerInputKeys::ABILITY_1,
            PlayerInputKeys::ABILITY_2,
            PlayerInputKeys::ABILITY_3,
            PlayerInputKeys::ABILITY_4,
        ]
    }
    pub fn set_left_click(&mut self, clicked: bool) {
        self.binary_inputs.set(PlayerInputKeys::LEFT_CLICK, clicked);
    }
    pub fn set_right_click(&mut self, clicked: bool) {
        self.binary_inputs
            .set(PlayerInputKeys::RIGHT_CLICK, clicked);
    }
    pub fn left_click(&self) -> bool {
        self.pressed(PlayerInputKeys::LEFT_CLICK)
    }
    pub fn right_click(&self) -> bool {
        self.pressed(PlayerInputKeys::RIGHT_CLICK)
    }

    pub fn pressed(&self, key: PlayerInputKeys) -> bool {
        self.binary_inputs.contains(key)
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

impl std::fmt::Debug for PlayerInput {
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

impl std::fmt::Debug for Radians {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{value:.precision$?}°",
            precision = f.precision().unwrap_or(2),
            value = self.0 * 360.0 / std::f32::consts::TAU,
        ))
    }
}
