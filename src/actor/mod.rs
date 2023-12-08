use std::{collections::HashMap, time::Duration};

use crate::{
    ability::crowd_control::{CCMap, CCType},
    game_manager::{InGameSet, TEAM_1},
    prelude::*,
};
use oxidized_navigation::NavMeshAffector;

use self::{controller::*, minion::MinionPlugin, player::*};

pub mod controller;
pub mod minion;
pub mod player;

pub struct ActorPlugin;
impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        //Resources
        app.insert_resource(PlayerInput::default());
        app.register_type::<PlayerInput>()
            .register_type::<Stat>()
            .register_type::<Modifier>()
            .register_type::<AttributeTag>();

        //Plugins
        app.add_plugins((MinionPlugin, ControllerPlugin, PlayerPlugin));

        //Systems
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

#[derive(Component, Deref, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct PreviousActorState(ActorState);

pub fn update_previous_actor(mut actors: Query<(&mut PreviousActorState, &ActorState)>) {
    for (mut previous, current) in &mut actors {
        previous.0 = current.clone();
    }
}

#[derive(Component)]
pub struct HasHealthBar;

pub fn player_swivel(mut players: Query<(&mut Transform, &PlayerInput, &CCMap), With<Player>>) {
    for (mut player_transform, inputs, cc_map) in players.iter_mut() {
        if cc_map.map.contains_key(&CCType::Stun) {
            continue;
        }
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, inputs.yaw as f32).into();
    }
}

pub fn player_movement(mut query: Query<(&Attributes, &mut Controller, &PlayerInput, &CCMap)>) {
    for (attributes, mut controller, player_input, cc_map) in query.iter_mut() {
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

        controller.direction = movement_vector;
        controller.max_speed = speed;
    }
}
