use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

#[derive(Component, Debug)]
pub struct Homing(pub Entity);

pub fn track_homing(mut homing_query: Query<(&Homing, &mut Transform, &mut Velocity)>, targets: Query<&GlobalTransform, Without<Homing>>) {
    for (homing_on, mut homing_transform, mut velocity) in homing_query.iter_mut() {
        let Ok(transform_to_track) = targets.get(homing_on.0) else { return };
        let speed = 20.0;
        homing_transform.look_at(transform_to_track.translation(), Vec3::Y);
        let direction = homing_transform.rotation * -Vec3::Z;
        *velocity = Velocity {
            linvel: direction * speed,
            ..default()
        };
    }
}
