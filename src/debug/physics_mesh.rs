use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_xpbd_3d::{
    parry::shape::{TriMesh, TypedShape},
    prelude::*,
};

#[derive(Component, Copy, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct PhysicsDebugMesh;

pub trait AsMesh {
    fn as_meshes(&self) -> Vec<(Mesh, Transform)>;
}

pub fn trimesh_to_mesh(trimesh: &TriMesh) -> Mesh {
    let points = trimesh.vertices();
    let indices = trimesh.indices();
    let points: Vec<[f32; 3]> = points
        .iter()
        .map(|point| [point.x, point.y, point.z])
        .collect();
    let indices: Vec<u32> = indices.iter().flatten().cloned().collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}

impl<'a> AsMesh for TypedShape<'a> {
    fn as_meshes(&self) -> Vec<(Mesh, Transform)> {
        let mut meshes = Vec::new();
        match self {
            TypedShape::Ball(ball) => {
                let mesh = Mesh::from(shape::UVSphere {
                    radius: ball.radius,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cuboid(cuboid) => {
                let dim = cuboid.half_extents * 2.0;
                let mesh = Mesh::from(shape::Box::new(dim.x, dim.y, dim.z));
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Capsule(capsule) => {
                let a: Vec3 = capsule.segment.a.into();
                let b: Vec3 = capsule.segment.b.into();
                let midpoint = a * 0.5 + b * 0.5;
                let length = (a - b).length();
                let mesh = Mesh::from(shape::Capsule {
                    depth: length,
                    radius: capsule.radius,
                    ..default()
                });
                meshes.push((
                    mesh,
                    Transform {
                        translation: midpoint,
                        ..default()
                    },
                ));
            }
            TypedShape::Segment(_segment) => {}
            TypedShape::Triangle(_triangle) => {}
            TypedShape::TriMesh(trimesh) => {
                let mesh = trimesh_to_mesh(trimesh);
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Polyline(_polyline) => {}
            TypedShape::HalfSpace(_halfspace) => {
                //let dir = halfspace.normal();
            }
            TypedShape::HeightField(height_field) => {
                let (points, indices) = height_field.to_trimesh();
                let trimesh = TriMesh::new(points, indices);
                let mesh = trimesh_to_mesh(&trimesh);
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Compound(compound) => {
                for (isometry, shape) in compound.shapes() {
                    let compound_transform = Transform {
                        translation: isometry.translation.into(),
                        rotation: isometry.rotation.into(),
                        scale: Vec3::ONE,
                    };

                    let typed_shape = shape.as_typed_shape();
                    for (mesh, transform) in typed_shape.as_meshes() {
                        let transform = compound_transform * transform;
                        meshes.push((mesh, transform));
                    }
                }
            }
            TypedShape::ConvexPolyhedron(convex_polyhedron) => {
                let (points, indices) = convex_polyhedron.to_trimesh();
                let trimesh = TriMesh::new(points, indices);
                let mesh = trimesh_to_mesh(&trimesh);
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cylinder(cylinder) => {
                let mesh = Mesh::from(shape::Cylinder {
                    radius: cylinder.radius,
                    height: cylinder.half_height * 2.0,
                    ..default()
                });
                meshes.push((mesh, Transform::default()));
            }
            TypedShape::Cone(_cone) => {}
            TypedShape::RoundCuboid(_round_cuboid) => {}
            TypedShape::RoundTriangle(_round_triangle) => {}
            TypedShape::RoundCylinder(_round_cylinder) => {}
            TypedShape::RoundCone(_round_cone) => {}
            TypedShape::RoundConvexPolyhedron(_round_convex_polyhedron) => {}
            _ => {},
        };

        meshes
    }
}

/// If the collider has changed, then produce a new debug mesh for it.
pub fn init_physics_meshes(
    mut commands: Commands,
    //ctx: Res<RapierContext>,
    materials: Query<&Handle<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    colliders: Query<
        (Entity, &Collider, Option<&ColliderParent>),
        (Changed<Collider>, Without<Sensor>),
    >,
    childrens: Query<&Children>,
    physics_mesh: Query<&PhysicsDebugMesh>,
    mut removed: RemovedComponents<Collider>,
) {
    for entity in removed.read() {
        if let Ok(children) = childrens.get(entity) {
            for child in children.iter() {
                if physics_mesh.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
    }

    for (entity, collider, collider_parent) in &colliders {
        let material = if let Some(parent) = collider_parent {
            materials.get(parent.get()).ok()
        } else {
            materials.get(entity).ok()
        };

        let material = material.cloned().unwrap_or(Default::default());

        if let Ok(children) = childrens.get(entity) {
            for child in children.iter() {
                if physics_mesh.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        let collider_meshes = collider.shape().as_typed_shape().as_meshes();
        if collider_meshes.len() > 0 {
            let physics_meshes = commands
                .spawn(SpatialBundle::default())
                .insert(PhysicsDebugMesh)
                //.insert(DebugVisible)
                .insert(Name::new("Physics debug meshes"))
                .id();

            commands
                .entity(entity)
                .insert(VisibilityBundle::default())
                .add_child(physics_meshes);

            for (mesh, transform) in collider_meshes {
                let handle = meshes.add(mesh);
                commands
                    .spawn(PbrBundle {
                        mesh: handle,
                        transform: transform,
                        material: material.clone(),
                        ..default()
                    })
                    .insert(PhysicsDebugMesh)
                    //.insert(DebugVisible)
                    .insert(Name::new("Physics debug mesh"))
                    .set_parent(physics_meshes);
            }
        }
    }
}
