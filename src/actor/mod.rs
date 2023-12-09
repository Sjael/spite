

use std::time::{Instant, Duration};

use crate::{
    ability::{crowd_control::{CCMap, CCType}, cast::IncomingDamageLog},
    prelude::*,
    session::director::InGameSet,
    stats::Bounty, ui::scoreboard::Scoreboard,
};

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
        //app.add_systems(Last, despawn_dead.run_if(in_state(GameState::InGame)));
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


fn give_kill_credit(
    changed_states: Query<(Option<&Bounty>, &ActorState, &IncomingDamageLog), Changed<ActorState>>,
    mut victors: Query<(&mut Attributes, &ActorType)>,
    mut scoreboard: ResMut<Scoreboard>,
){
    const TIME_FOR_KILL_CREDIT: u64 = 30;
    for (bounty, state, log) in changed_states.iter(){
        if state == &ActorState::Alive { continue }
        
        let mut killers = Vec::new();
        for instance in log.list.iter().rev() {
            if Instant::now().duration_since(instance.when)
                > Duration::from_secs(TIME_FOR_KILL_CREDIT)
            {
                break;
            }
            //let Ok(attacker) = the_guilty.get(instance.attacker) else {continue};
            killers.push(instance.attacker);
        }
        for (index, awardee) in killers.iter().enumerate() {
            let Ok((mut attributes, awardee_actor)) = victors.get_mut(*awardee) else {
                continue;
            };
    
            if let Some(bounty) = bounty {
                let gold = attributes.get_mut(Stat::Gold);
                *gold += bounty.gold;
                let xp = attributes.get_mut(Stat::Xp);
                *xp += bounty.xp;
            }
    
            if let ActorType::Player(killer) = awardee_actor {
                let killer_scoreboard = scoreboard.0.entry(*killer).or_default();
                if index == 0 {
                    killer_scoreboard.kda.kills += 1;
                } else {
                    killer_scoreboard.kda.assists += 1;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct DeathDelay(pub Timer);

fn start_death_timer(
    mut commands: Commands,
    changed_states: Query<(Entity, &ActorState), Changed<ActorState>>,
){
    let respawn_timer = 8.0;
    for (entity, state) in changed_states.iter(){
        if state == &ActorState::Alive { continue }
        // Add respawn timer to director

        // Add despawn delay component to dead thing
        let death_delay = DeathDelay(Timer::from_seconds(10.0, TimerMode::Once));
        commands.entity(entity).insert(death_delay);
    }
}


fn tick_death_timer(
    mut respawning: Query<(Entity, &mut DeathDelay, &mut Visibility)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut timer, mut vis) in respawning.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.percent() > 0.7 {
            *vis = Visibility::Hidden;
        }
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// probably want to change this respawn system to just despawn the entity instead of hiding it
//
// it makes sense to just move players back to spawn at first, but what about minions? 
// the hiding + moving way works for players cus u will have just 1
// like is there supposed to be a cap for minions then? we def despawn them so i think having a unified way of respawning thing too
// and should do the same for players, fully despawning and setting a respawn in something managed by the director instead of component
// fn respawn_entity(
//     mut commands: Commands,
//     mut the_damned: Query<(&mut Visibility, &ActorState, Entity, &ActorType), Changed<ActorState>>,
//     local_player: Res<Player>,
//     //mut spectate_events: EventWriter<SpectateEvent>,
// ) {
//     for (mut vis, state, entity, actor_type) in the_damned.iter_mut(){
//         if state == ActorState::Dead { continue }
//         commands.entity(entity).remove::<Respawn>();
//         *vis = Visibility::Visible;
//         if actor_type == ActorType::Player(*local_player) {
//             spectate_events.send(SpectateEvent {
//                 entity: event.entity,
//             });
//         }
//     }
// }
