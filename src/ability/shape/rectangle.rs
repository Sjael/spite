use bevy::{
    math::Vec3,
    render::{mesh::{Indices, Mesh}, render_resource::PrimitiveTopology},
};
use bevy_rapier3d::prelude::*;

pub struct Rectangle {
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

impl Rectangle {
    pub fn flat(length: f32, width: f32) -> Self {
        let positions = vec![
            [width / 2.0, 0.0, -length / 2.0],
            [width / 2.0, 0.0, length / 2.0],
            [-width / 2.0, 0.0, length / 2.0],
            [-width / 2.0, 0.0, -length / 2.0],
        ];

        // normals pointing up
        let indices = vec![
            [0, 2, 1],
            [0, 3, 2],
        ];
        Self { positions, indices }
    }

    pub fn extruded(length: f32, width: f32) -> Self {
        let flat = Rectangle::flat(length, width);
        const ABILITY_HEIGHT: f32 = 1.0;

        let mut extruded = Rectangle {
            positions: flat.positions.clone(),
            indices: flat.indices.clone(),
        };

        extruded.positions.extend(
            flat.positions.iter()
                .map(|position| [position[0], position[1] + ABILITY_HEIGHT , position[2]]),
        );

        // make top tris
        extruded.indices.extend(
            flat.indices.iter()
                .map(|tri| [tri[0] + 4, tri[1] + 4, tri[2] + 4]),
        );

        // make side tris
        for i in 0..=3 {
            extruded.indices.push([i + 4, i, i + 3]);
            extruded.indices.push([i, i + 4, i + 1]);
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

        // TODO change to Cuboid, not ConvexHull
        Collider::convex_hull(&vertices).unwrap()
    }
}
