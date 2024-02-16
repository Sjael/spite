use crate::{ability::TargetsInArea, actor::player::LocalPlayer, prelude::*, ui::spectating::FocusedHealthEntity};

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
                TargetsInArea::default(),
            ));
        });
    }
}

pub fn focus_objective_health(
    local_player: Option<Res<LocalPlayer>>,
    mut focused_health_entity: ResMut<FocusedHealthEntity>,
    targets_query: Query<(&TargetsInArea, &Parent), (Changed<TargetsInArea>, With<HealthBarDetect>)>,
) {
    let Some(local_player) = local_player else { return };
    for (targets, parent) in targets_query.iter() {
        if targets.list.contains(&**local_player) {
            focused_health_entity.0 = Some(parent.get());
        } else {
            focused_health_entity.0 = None;
        }
    }
}
