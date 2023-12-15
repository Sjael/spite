use std::collections::HashMap;

use bevy::prelude::*;

use homing::track_homing;

use crate::ability::Ability;
use crate::ability::AbilityTooltip;
use crate::ability::TickBehavior;

use self::non_damaging::*;
use self::queue::*;
use self::timeline::*;

#[derive(Component)]
pub struct ProcMap(pub HashMap<Ability, Vec<AbilityBehavior>>);

pub enum AbilityBehavior {
    Homing,
    OnHit,
}

pub struct AreaPlugin;
impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AbilityTooltip>();
        app.register_type::<TickBehavior>();
        app.add_event::<HealthChangeEvent>();
        app.add_event::<BuffEvent>();
        app.add_event::<CCEvent>();
        app.add_event::<AreaOverlapEvent>();

        app.add_systems(PreUpdate, (apply_interval, catch_collisions));
        app.add_systems(
            Update,
            (
                tick_area_lifetime,
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

pub mod homing;
pub mod non_damaging;
pub mod queue;
pub mod timeline;

/*
ullr axe
    TargetsInArea
    Tags
    max hits
    TargetsHittable

Ra heal
    TargetsInArea
    Tags
    Ticks individually
    TargetsHittable

Anubis 3
    TargetsInArea
    Tags
    Ticks statically
    TargetsHittable

Tower
    TargetsInArea
    Tags
    Ticks statically
    PausesWhenEmpty (ready to fire on enter) (doesnt reset unless it fires)
    TargetsHittable
    FilterTargets::Closest (1)

 */
