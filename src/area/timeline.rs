use std::time::Duration;

use bevy::prelude::*;

use crate::ability::{timeline::{AreaLifetime, AreaTimeline}, FiringInterval, TickBehavior};

pub fn tick_area_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut AreaLifetime, Entity)>,
) {
    for (mut lifetime, entity) in lifetimes.iter_mut() {
        //dbg!(lifetime.clone());
        lifetime.seconds -= time.delta_seconds_f64();
        if lifetime.seconds <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(super) fn tick_timeline(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(&mut AreaTimeline, Entity, &mut Visibility)>,
) {
    for (mut timeline, entity, mut vis) in lifetimes.iter_mut() {
        //dbg!(lifetime.clone());
        timeline.timer.tick(time.delta());
        if timeline.timer.finished() {
            use crate::ability::timeline::DeployAreaStage::*;
            let new_stage = timeline.stage.get_next_stage();
            let new_time = timeline.blueprint.get(&new_stage).unwrap_or(&0.0).clone();
            timeline.timer = Timer::new(Duration::from_secs_f32(new_time), TimerMode::Once);
            if timeline.stage == Recovery {
                commands.entity(entity).despawn_recursive();
            } else if timeline.stage == Windup {
                *vis = Visibility::Visible;
            }
        }
    }
}

pub fn apply_interval(
    mut area_timers: Query<(&FiringInterval, &mut TickBehavior), Added<TickBehavior>>,
) {
    for (interval, mut tick_behavior) in &mut area_timers {
        match *tick_behavior {
            TickBehavior::Static(ref mut static_timer) => {
                static_timer.set_duration(Duration::from_secs_f32(interval.0));
            }
            _ => (),
        }
    }
}
