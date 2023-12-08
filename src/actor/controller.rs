use crate::prelude::*;

pub struct ControllerPlugin;
impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Controller>();

        app.add_systems(FixedUpdate, controller_movement.in_set(InGameSet::Update));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Controller {
    pub max_speed: f32,
    pub direction: Vec3,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            max_speed: 1.0,
            direction: Vec3::ZERO,
        }
    }
}

impl Controller {
    pub fn goal_velocity(&self) -> Vec3 {
        self.direction.clamp_length_max(1.0) * self.max_speed
    }
}

pub fn controller_movement(
    time: Res<Time>,
    mut controllers: Query<(
        &mut ExternalImpulse,
        &mut LinearVelocity,
        &Mass,
        &Controller,
    )>,
) {
    let dt = time.delta_seconds();

    for (mut impulse, mut velocity, mass, controller) in &mut controllers {
        let mass = mass.0;

        let goal_velocity = controller.goal_velocity();
        let displacement = goal_velocity - **velocity;
        let movement_impulse = displacement * mass;
        //info!("displacement: {:?}", displacement);

        /*
        let friction_velocity = velocity;
        let friction_align = goal_align;
        let friction_offset = friction_align.clamp(0.0, goal_vel.length());
        friction_velocity -= friction_offset * goal_dir;

        let friction_strength = mass / dt;
        let friction_force = friction_velocity * friction_strength;
        */

        /*
        let squish = 0.2;
        gizmos.ray(Vec3::ZERO, goal_vel * squish, Color::GREEN);
        gizmos.sphere(goal_align * goal_dir * squish, Quat::IDENTITY, 0.1, Color::GREEN);
        gizmos.ray(Vec3::ZERO, relative_velocity * squish, Color::BLUE);
        gizmos.ray(relative_velocity * squish, -goal_offset * goal_dir * squish, Color::RED);
        gizmos.ray(Vec3::new(0.0, 0.1, 0.0), friction_velocity * squish, Color::CYAN);
        */
        if movement_impulse.length() > 0.05 {
            //info!("disp: {:.2?}", displacement);
            //info!("move: {:.2?}", movement_impulse);
        }

        if velocity.length() > 0.0 {
            info!("velo: {:.2?}", velocity);
        }

        //**velocity = goal_velocity;
        impulse.apply_impulse(movement_impulse);
    }
}
