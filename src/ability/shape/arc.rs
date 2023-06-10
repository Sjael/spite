use bevy::{
    math::Vec3,
    render::{mesh::{Indices, Mesh}, render_resource::PrimitiveTopology},
};

use bevy_rapier3d::prelude::*;

pub struct Arc {
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

impl Arc {
    pub fn flat(radius: f32, angle: f32) -> Self {
        let nb_points = (angle / 30.).floor().abs() as u32;

        let mut positions = vec![[0., 0., 0.]];
        let mut indices = Vec::new();
        for i in 0..=nb_points {
            if i > 0 {
                indices.push([0, i, i + 1]);
            }

            let angle_point =
                (i as f32 * angle / (nb_points as f32) - 90. + angle / 2.0).to_radians();
            positions.push([angle_point.cos() * radius, 0.0, angle_point.sin() * radius]);
        }

        Self { positions, indices }
    }

    pub fn extruded(radius: f32, angle: f32) -> Self {
        let flat = Arc::flat(radius, angle);

        // Extrude it out upwards.
        let mut extruded = Arc {
            positions: flat.positions.clone(),
            indices: flat.indices.clone(),
        };

        // Generate top extrusion vertices.
        extruded.positions.extend(
            flat.positions
                .iter()
                .map(|position| [position[0], position[1] + 1., position[2]]),
        );

        // Generate top extrusion indices.
        extruded.indices.extend(
            flat.indices
                .iter()
                .map(|tri| tri.map(|index| index + flat.positions.len() as u32))
                .map(|tri| [tri[0], tri[2], tri[1]]), // we swizzle these to make it flip the triangle's face
        );

        // Generate side faces.
        let half_positions = flat.positions.len() as u32 - 1;
        for i in 0..=half_positions {
            extruded
                .indices
                .push([i + half_positions + 1, i, i + half_positions]);
            extruded.indices.push([i, i + half_positions + 1, i + 1]);
        }

        extruded
    }

    pub fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let normals = std::iter::repeat([0.0, 1.0, 0.0])
            .take(self.positions.len())
            .collect::<Vec<_>>();
        let uvs = std::iter::repeat([0.0, 0.0])
            .take(self.positions.len())
            .collect::<Vec<_>>();

        mesh.set_indices(Some(Indices::U32(
            self.indices
                .clone()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        )));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions
                .iter()
                .map(|position| [position[0], position[1], position[2]])
                .collect::<Vec<_>>(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
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
