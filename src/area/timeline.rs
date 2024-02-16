use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;

pub(super) fn tick_timeline(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut AreaTimeline, Entity, &mut Visibility)>,
) {
    for (mut timeline, entity, mut vis) in lifetimes.iter_mut() {
        timeline.tick(time.delta());
        if timeline.stage == CastStage::Despawn {
            commands.entity(entity).despawn_recursive();
        } else if timeline.stage == CastStage::Firing {
            *vis = Visibility::Visible;
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Component)]
pub struct AreaTimeline {
    pub stage: CastStage,
    pub timer: Timer,
    pub blueprint: HashMap<CastStage, f32>,
}

impl AreaTimeline {
    pub fn new_at_stage(bp: HashMap<CastStage, f32>, stage: CastStage) -> Self {
        let x = bp.get(&stage).unwrap_or(&0.0).clone();
        Self {
            blueprint: bp,
            stage,
            timer: Timer::new(Duration::from_secs_f32(x), TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.finished() && self.stage != CastStage::Despawn {
            let new_stage = self.stage.get_next_stage();
            // dbg!(new_stage.clone());
            let new_time = self.blueprint.get(&new_stage).unwrap_or(&0.0).clone();
            self.stage = new_stage;
            self.timer = Timer::new(Duration::from_secs_f32(new_time), TimerMode::Once);
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum CastStage {
    Input,  // Player hit key on instacast, or left-clicked normal cast
    Casted, // animation + sound queues went off without getting cc'd
    // Spawned into world here (not currently but ideally)
    Windup,   // Placed in the world, but has delay so enemies can react
    Firing,   // active in world, applying tags
    Spindown, // delay before despawn
    Despawn,
}

impl CastStage {
    pub fn get_next_stage(&mut self) -> Self {
        use CastStage::*;
        match self {
            Input => Casted,
            Casted => Windup,
            Windup => Firing,
            Firing => Spindown,
            _ => Despawn,
        }
    }
}

impl Default for CastStage {
    fn default() -> Self {
        Self::Input
    }
}

// #[derive(Component)]
// pub struct EnemyVisDelay(pub f32);
