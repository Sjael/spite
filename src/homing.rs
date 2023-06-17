
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Homing(pub Entity);

fn track_homing(
    mut homing_query: Query<(&Homing, &mut Transform)>,
    transform_query: Query<&Transform, Without<Homing>>,
){
    for (homing_on, mut homing_transform) in homing_query.iter_mut(){
        let Ok(transform_to_track) = transform_query.get(homing_on.0) else {return};
        homing_transform.look_at(transform_to_track.translation, Vec3::Y);
    }
}