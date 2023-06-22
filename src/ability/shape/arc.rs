use bevy::{
    math::Vec3,
    render::{mesh::{Indices, Mesh}, render_resource::PrimitiveTopology},
};

use bevy_rapier3d::prelude::*;


use super::cross_product;

pub struct Arc {
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

impl Arc {
    pub fn flat(radius: f32, angle: f32) -> Self {
        let mut positions = vec![[0.,0.,0.]]; // center of arc point
        let mut indices = Vec::new();
        const ROTATION_OFFSET: f32 = 90.0; // just an engine thing
        const DEGREES_PER_VERTEX: u32 = 15;

        let segments = (angle / DEGREES_PER_VERTEX as f32).ceil().abs() as u32;
        let a_circle = angle == 360.0;
        let mut additional_point = 0;
        if !a_circle{
            additional_point += 1; // need another point if not a circle
        }

        let points_on_arc = segments + additional_point;

        for point_index in 0..points_on_arc{
            if point_index > 0 {
                // always pushing 0 for center of arc point, makes a fan of tris
                indices.push([0, point_index + 1, point_index]);
            }

            let starting_angle = -angle / 2.0 - ROTATION_OFFSET;
            let angle_of_point = starting_angle + (angle * point_index as f32 / segments as f32);
            positions.push([
                angle_of_point.to_radians().cos() * radius, 
                0.0, 
                angle_of_point.to_radians().sin() * radius
            ]);
        }
        if a_circle{            
            indices.push([0, 1, points_on_arc]);
        }

        Self { positions, indices }
    }

    pub fn extruded(radius: f32, angle: f32) -> Self {
        let flat = Arc::flat(radius, angle);
        const ABILITY_HEIGHT: f32 = 1.0;
        let is_circle = angle == 360.0;

        let mut extruded = Arc {
            positions: flat.positions.clone(),
            indices: flat.indices.clone(),
        };

        // Generate top extrusion vertices and indices.
        extruded.positions.extend(
            flat.positions
                .iter()
                .map(|position| [position[0], position[1] + ABILITY_HEIGHT, position[2]]),
        );
        extruded.indices.extend(
            flat.indices
                .iter()
                .map(|tri| tri.map(|index| index + flat.positions.len() as u32))
                //.map(|tri| [tri[0], tri[1], tri[2]]),
        );

        let total_points = flat.positions.len() as u32;

        // draw sides along arc
        for current_point in 1..total_points {
            let mut next_point = current_point + 1;
            if current_point == total_points - 1{
                if is_circle {
                    // IF last point and circle, then loop next point to the 1st index
                    next_point = 1;
                } else{
                    // ELSE draw sides from center and skip
                    extruded.indices.push([0, total_points + 1, 1]);
                    extruded.indices.push([0, total_points, total_points + 1]);
                    extruded.indices.push([0, total_points - 1, total_points * 2 - 1]);
                    extruded.indices.push([0, total_points * 2 - 1, total_points]);
                    continue;
                }
            }
            extruded.indices.push([current_point, current_point + total_points, next_point]);
            extruded.indices.push([next_point, current_point + total_points, next_point + total_points]);
        }

        extruded
    }

    pub fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions
                .iter()
                .map(|position| [position[0], position[1], position[2]])
                .collect::<Vec<_>>(),
        );
        mesh.set_indices(Some(Indices::U32(
            self.indices
                .clone()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        )));

        // Face normal calculation
        // bevy only does vertex normals right now so we apply to every vertex for sharp shading
        
        let normals2 = self.indices.iter()
            .map(|tri| {
                let origin: Vec3 = self.positions[tri[0] as usize].into();
                let p1: Vec3 = self.positions[tri[1] as usize].into();
                let p2: Vec3 = self.positions[tri[2] as usize].into();
                let normal = cross_product(p1-origin, p2-origin).normalize();
            });
         

        let normals = std::iter::repeat([0.0, 1.0, 0.0])
            .take(self.positions.len())
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        let uvs = std::iter::repeat([0.0, 0.0])
            .take(self.positions.len())
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }

    pub fn collider(&self) -> Collider {
        let vertices = self
            .positions
            .iter()
            .map(|position| Vec3::from(*position))
            .collect::<Vec<_>>();

        Collider::convex_hull(&vertices).unwrap()
    }
}
