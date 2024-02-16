use std::collections::HashMap;

use bevy::prelude::*;
use homing::track_homing;

use crate::{
    ability::Ability,
    area::{non_damaging::*, queue::*, timeline::*},
};

pub struct AreaPlugin;
impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthChangeEvent>();
        app.add_event::<BuffEvent>();
        app.add_event::<CCEvent>();
        app.add_event::<AreaOverlapEvent>();

        app.add_systems(PreUpdate, catch_collisions);
        app.add_systems(
            Update,
            (
                tick_hit_timers,
                track_homing,
                add_health_bar_detect_colliders,
                focus_objective_health,
                tick_timeline.before(area_queue_targets),
            ),
        );
        app.add_systems(
            Update,
            (
                filter_targets,
                area_queue_targets,
                area_apply_tags,
                despawn_after_max_hits,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
pub struct Fountain;

#[derive(Component)]
pub struct ProcMap(pub HashMap<Ability, Vec<AbilityBehavior>>);

pub enum AbilityBehavior {
    Homing,
    OnHit,
}

pub mod homing;
pub mod non_damaging;
pub mod queue;
pub mod timeline;
