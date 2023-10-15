use bevy::prelude::*;
use derive_more::{Deref, DerefMut, Display};
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};
use std::collections::BTreeMap;

use crate::{ability::Ability, actor::player::HoveredAbility, ui::mouse::MouseState, GameState};

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<Slot>::default(),
            InputManagerPlugin::<Ability>::default(),
        ));

        app.insert_resource(MouseSensitivity(1.0));

        app.add_systems(OnEnter(GameState::InGame), clean_inputs);
        app.add_systems(
            PreUpdate,
            (copy_action_state
                .after(InputManagerSystem::ManualControl)
                .run_if(in_state(GameState::InGame)),),
        );
        app.add_systems(Update, report_abilities_used);
    }
}

fn clean_inputs(mut query: Query<&mut ActionState<Slot>>) {
    for mut slot_state in &mut query {
        slot_state.consume_all();
    }
}
// There are 3 layers of data (Keys -> Slots -> Abilities)
// This is the system passing on actions from first 2 layers on to final layer
// We can now have ability 'slots' in our game, like an actionbar in WoW or
// Diablo that we can drag stuff onto soon tm
pub fn copy_action_state(
    mut query: Query<(
        &ActionState<Slot>,
        &mut ActionState<Ability>,
        &SlotAbilityMap,
        &HoveredAbility,
    )>,
    mouse_is_free: Res<State<MouseState>>,
) {
    for (slot_state, mut ability_state, slot_ability_map, hovered) in query.iter_mut() {
        for slot in Slot::variants() {
            // skip auto attack if we are in a menu
            if slot == Slot::LeftClick
                && (*mouse_is_free == MouseState::Free || hovered.0.is_some())
            {
                continue
            }
            if let Some(&matching_ability) = slot_ability_map.get(&slot) {
                // Even copies information about how long the buttons have been pressed or
                // released
                ability_state.set_action_data(
                    matching_ability.clone(),
                    slot_state.action_data(slot).clone(),
                );
            }
        }
    }
}

fn report_abilities_used(query: Query<&ActionState<Ability>>) {
    for ability_state in query.iter() {
        for _ability in ability_state.get_just_pressed() {
            //dbg!(ability);
        }
    }
}

// These are the things that dont change from users, but they can rebind keys to
// them, and the abilities they call
#[allow(dead_code)]
#[derive(
    Actionlike, Reflect, Clone, Copy, Debug, Display, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
pub enum Slot {
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    LeftClick,
    RightClick,
}

// This struct stores which ability corresponds to which slot for a particular
// player aka their Loadout
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct SlotAbilityMap {
    pub map: BTreeMap<Slot, Ability>,
}

impl SlotAbilityMap {
    // Default loadout
    pub fn new() -> Self {
        let mut ability_slot_map = SlotAbilityMap::default();
        ability_slot_map.insert(Slot::Ability1, Ability::Frostbolt);
        ability_slot_map.insert(Slot::Ability2, Ability::Fireball);
        ability_slot_map.insert(Slot::Ability3, Ability::Bomb);
        ability_slot_map.insert(Slot::LeftClick, Ability::BasicAttack);
        ability_slot_map
    }
}

// Could just remove this and put the actionstates in the fn return above?
// Bundles seem kinda pointless until you have multiple ways of spawning things
#[derive(Bundle, Default)]
pub struct SlotBundle {
    pub input_slot_map: InputMap<Slot>,
    pub slot_action_state: ActionState<Slot>,
    pub slot_ability_map: SlotAbilityMap,
    pub ability_action_state: ActionState<Ability>,
}

impl SlotBundle {
    // These are the placeholder keybinds
    pub fn new() -> Self {
        let input_slot_map = InputMap::new([
            (KeyCode::Key1, Slot::Ability1),
            (KeyCode::Key2, Slot::Ability2),
            (KeyCode::Key3, Slot::Ability3),
        ])
        .insert(MouseButton::Left, Slot::LeftClick)
        .insert(MouseButton::Right, Slot::RightClick)
        .build();

        SlotBundle {
            input_slot_map,
            slot_ability_map: SlotAbilityMap::new(),
            ..default()
        }
    }
}
