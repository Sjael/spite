pub mod arc;
pub mod rectangle;

pub use arc::*;
use bevy_rapier3d::prelude::Collider;
pub use rectangle::*;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[reflect_value(Component, PartialEq, Serialize, Deserialize)]
pub enum AbilityShape {
    Arc { radius: f32, angle: f32 },
    Rectangle { length: f32, width: f32 },
}

impl Default for AbilityShape {
    fn default() -> Self {
        AbilityShape::Rectangle {
            length: 2.0,
            width: 3.0,
        }
    }
}

impl AbilityShape {
    pub fn load(self) -> (Mesh, Collider) {
        match self {
            AbilityShape::Arc { radius, angle } => {
                let arc = Arc::extruded(radius, angle);
                (arc.mesh(), Collider::ball(radius))
            }
            AbilityShape::Rectangle { length, width } => {
                let rect = Rectangle::flat(length, width);
                (rect.mesh(), rect.collider())
            }
        }
    }
}

pub fn load_ability_shape(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &AbilityShape), Added<AbilityShape>>,
) {
    for (entity, shape) in query.iter() {
        let (mesh, collider_shape) = shape.clone().load();
        commands.entity(entity).insert((
            meshes.add(mesh),
            materials.add(Color::rgb(0.1, 0.2, 0.7).into()),
            Visibility::default(),
            ComputedVisibility::default(),
            collider_shape,
        ));
    }
}

pub fn cross_product(first: Vec3, second: Vec3) -> Vec3{
    let x = first[1] * second[2] - second[1] * first[2];
    let y = first[2] * second[0] - second[2] * first[0];
    let z = first[0] * second[2] - second[0] * first[1];
    Vec3::new(x,y,z)
    // normalize if you want generated normal
}