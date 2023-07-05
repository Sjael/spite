pub mod arc;
pub mod rectangle;

pub use arc::*;
use bevy_rapier3d::prelude::Collider;
pub use rectangle::*;
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::assets::MaterialPresets;

use super::bundles::Targetter;

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
                (arc.mesh(), Collider::cylinder(0.5, radius))
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
    query: Query<(Entity, &AbilityShape, Option<&Targetter>), Added<AbilityShape>>,
    presets: Res<MaterialPresets>,
) {
    for (entity, shape, targetter) in query.iter() {
        let (mesh, collider_shape) = shape.clone().load();
        commands.entity(entity).insert((
            meshes.add(mesh),
            Visibility::default(),
            ComputedVisibility::default(),
            collider_shape,
        ));
        let new_material = presets.0.get("red").unwrap_or(&materials.add(Color::rgb(0.9, 0.2, 0.2).into())).clone();
        if let None = targetter{
            commands.entity(entity).insert(new_material);
        }
    }
}


pub fn cross_product(first: Vec3, second: Vec3) -> Vec3{
    let x = first[1] * second[2] - second[1] * first[2];
    let y = first[2] * second[0] - second[2] * first[0];
    let z = first[0] * second[2] - second[0] * first[1];
    Vec3::new(x,y,z)
    // normalize if you want generated normal
}