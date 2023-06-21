
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

#[derive(Component, Debug)]
pub struct Homing(pub Entity);

pub fn track_homing(
    mut homing_query: Query<(&Homing, &mut Transform, &mut Velocity)>,
    transform_query: Query<&GlobalTransform, Without<Homing>>,
){
    for (homing_on, mut homing_transform, mut velocity) in homing_query.iter_mut(){
        let Ok(transform_to_track) = transform_query.get(homing_on.0) else {return};
        homing_transform.look_at(transform_to_track.translation(), Vec3::Y);
        let direction = homing_transform.rotation * -Vec3::Z;
        *velocity = Velocity{
            linvel: direction * 20.0,
            ..default()
        };
    }
}