use std::time::Duration;

use crate::{
    actor::player::{Player, Players},
    prelude::*,
};

pub struct TeamPlugin;
impl Plugin for TeamPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Team>()
            .register_type::<TeamRoster>()
            .register_type::<Option<Timer>>()
            .register_type::<TeamRespawn>();

        app.add_systems(FixedUpdate, team_respawn.in_set(InGameSet::Pre));
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Component, Reflect)]
pub struct Team(pub TeamMask);

// Team masks
bitflags::bitflags! {
    #[derive(Reflect, Default,)]
    pub struct TeamMask: u32 {
        const ALL = 1 << 0;
        const TEAM_1 = 1 << 1;
        const TEAM_2 = 1 << 2;
        const TEAM_3 = 1 << 3;
        const NEUTRALS = 1 << 4;
    }
}

// Team Components
pub const TEAM_1: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_1.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_2: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_2.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_3: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::TEAM_3.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_NEUTRAL: Team = Team(TeamMask::from_bits_truncate(
    TeamMask::NEUTRALS.bits() | TeamMask::ALL.bits(),
));
pub const TEAM_ALL: Team = Team(TeamMask::from_bits_truncate(TeamMask::ALL.bits()));

/// Teams respawn as a group.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct TeamRespawn {
    timer: Option<Timer>,
    start_time: Duration,
}

impl TeamRespawn {
    /// Enable team group respawning and respawn at this rate when a player dies.
    pub fn new(start_time: Duration) -> Self {
        Self {
            timer: None,
            start_time,
        }
    }
}

pub fn team_respawn(
    time: Res<Time>,
    players: ResMut<Players>,
    mut teams: Query<(&TeamRoster, &mut TeamRespawn)>,
    mut actor_states: Query<&mut ActorState>,
) {
    for (roster, mut respawn_timer) in &mut teams {
        match &mut respawn_timer.timer {
            Some(timer) => {
                timer.tick(time.delta());

                if !timer.just_finished() {
                    continue;
                }

                for player in &roster.players {
                    let Some(entity) = players.get(player) else {
                        continue;
                    };
                    if let Ok(mut actor_state) = actor_states.get_mut(entity) {
                        if actor_state.clone() == ActorState::Dead {
                            *actor_state = ActorState::Alive;
                        }
                    }
                }

                respawn_timer.timer = None;
            }
            None => {
                for player in &roster.players {
                    let Some(entity) = players.get(player) else {
                        continue;
                    };
                    if let Ok(actor_state) = actor_states.get(entity) {
                        if *actor_state == ActorState::Dead {
                            respawn_timer.timer =
                                Some(Timer::new(respawn_timer.start_time, TimerMode::Once));
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Default, Deref, DerefMut, Reflect)]
pub struct TeamRoster {
    pub players: Vec<Player>,
}

impl TeamRoster {
    pub fn new(players: Vec<Player>) -> Self {
        Self { players }
    }
}
