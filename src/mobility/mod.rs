use bevy::prelude::*;

#[derive(Component)]
pub enum Mobility {
    Dash
}

#[derive(Component)]
pub struct MoveToTarget {
    pub transform: Transform,
    pub timer: Timer,
}

// fn move_mobility(mut query: Query<(&mut Transform, &mut MoveToTarget)>, time: Res<Time>) {
//     for (mut transform, mut move_to_target) in query.iter_mut() {
//         move_to_target.timer.tick(time.delta());
//         transform
//     }
// }
