use bevy::prelude::*;
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, Sensor};

use crate::{ability::TargetsInArea, actor::player::Player, ui::spectating::FocusedHealthEntity};

use super::{AreaOverlapEvent, AreaOverlapType};

#[derive(Component)]
pub struct HealthBarDetect;

#[derive(Component)]
pub struct ObjectiveHealthOwner {
    pub not_looking_range: f32,
    pub looking_range: f32,
}

pub fn add_health_bar_detect_colliders(
    detectors: Query<(Entity, &ObjectiveHealthOwner), Added<ObjectiveHealthOwner>>,
    mut commands: Commands,
) {
    for (entity, health_range) in &detectors {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Collider::cylinder(0.5, health_range.not_looking_range),
                HealthBarDetect,
                Sensor,
                Name::new("health bar detection range"),
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
                TargetsInArea::default(),
            ));
        });
    }
}

pub fn focus_objective_health(
    query: Query<&Parent, With<HealthBarDetect>>,
    players: Query<&Player>,
    local_player: Res<Player>,
    mut focused_health_entity: ResMut<FocusedHealthEntity>,
    mut area_events: EventReader<AreaOverlapEvent>,
) {
    for event in area_events.iter() {
        let Ok(parent) = query.get(event.sensor) else { continue };
        let Ok(player) = players.get(event.target) else { continue };
        if *player == *local_player {
            if event.overlap == AreaOverlapType::Entered {
                focused_health_entity.0 = Some(parent.get());
            } else {
                focused_health_entity.0 = None;
            }
        }
    }
}
