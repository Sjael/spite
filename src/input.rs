
use bevy::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use std::collections::BTreeMap;
use derive_more::{Deref, DerefMut, Display};

use crate::{ability::Ability, ui::mouse::MouseState};


pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Slot>::default())
           .add_plugin(InputManagerPlugin::<Ability>::default());

        app.add_systems((
            copy_action_state
                .in_base_set(CoreSet::PreUpdate)
                .after(InputManagerSystem::ManualControl),
            report_abilities_used,
        ));
    }
}

// There are 3 layers of data (Keys -> Slots -> Abilities)
// This is the system passing on actions from first 2 layers on to final layer
// We can now have ability 'slots' in our game, like an actionbar in WoW or Diablo that we can drag stuff onto soon tm
fn copy_action_state(
    mut query: Query<(
        &ActionState<Slot>,
        &mut ActionState<Ability>,
        &SlotAbilityMap,
    )>,
    mouse_is_free: Res<State<MouseState>>,
) {
    for (slot_state, mut ability_state, ability_slot_map) in query.iter_mut() {
        for slot in Slot::variants() {
            // skip auto attack if we are in a menu
            if slot == Slot::LeftClick && mouse_is_free.0 == MouseState::Free{
                continue;
            }
            if let Some(matching_ability) = ability_slot_map.get(&slot) {
                // Even copies information about how long the buttons have been pressed or released
                ability_state
                    .set_action_data(matching_ability.clone(), slot_state.action_data(slot).clone());
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

// These are the things that dont change from users, but they can rebind keys to them, and the abilities they call
#[allow(dead_code)]
#[derive(Actionlike, Clone, Copy, Debug, Default, Display, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Slot {
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    LeftClick,
    #[default]
    RightClick,
}


// This struct stores which ability corresponds to which slot for a particular player
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct SlotAbilityMap {
    pub map: BTreeMap<Slot, Ability>,
}

impl SlotAbilityMap{
    pub fn new() -> Self{        
        let mut ability_slot_map = SlotAbilityMap::default();
        ability_slot_map.insert(Slot::Ability1, Ability::Frostbolt);
        ability_slot_map.insert(Slot::Ability2, Ability::Fireball);
        ability_slot_map.insert(Slot::Ability3, Ability::Dash);
        ability_slot_map.insert(Slot::LeftClick, Ability::BasicAttack);
        ability_slot_map
    }
}

// THIS IS A PLACEHOLDER -- need to clean this up, probably into impl blocks
pub fn setup_player_slots() -> impl Bundle{
    let input_slot_map = InputMap::new([
        (KeyCode::Key1, Slot::Ability1),
        (KeyCode::Key2, Slot::Ability2),
        (KeyCode::Key3, Slot::Ability3),
    ])
    .insert(MouseButton::Left, Slot::LeftClick)
    .insert(MouseButton::Right, Slot::RightClick)
    .build();

    SlotBundle{
        input_slot_map,
        slot_ability_map: SlotAbilityMap::new(),
        ..default()
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