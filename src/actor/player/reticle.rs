//! Targetting reticle

use crate::{actor::input::PlayerInput, prelude::*};

#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub cam_height: f32,
}

pub fn move_reticle(
    mut reticles: Query<(&mut Transform, &Reticle)>,
    player_input: Res<PlayerInput>,
) {
    for (mut transform, reticle) in &mut reticles {
        let current_angle = player_input.pitch.clamp(-1.57, 0.);
        transform.translation.z = (1.57 + current_angle).tan() * -reticle.cam_height;
        transform.translation.z = transform.translation.z.clamp(-reticle.max_distance, 0.);
    }
}
