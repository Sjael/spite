use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        fxaa::Fxaa,
        tonemapping::{DebandDither, Tonemapping},
    },
    transform::TransformSystem,
};

use crate::{
    actor::ActorState,
    game_manager::{InGameSet},
    prelude::*,
    ui::SpectatingSet,
};

use super::player::PlayerInput;

pub struct ViewPlugin;
impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpectatableObjects::default());

        app.add_event::<SpectateEvent>();
        app.add_event::<PossessEvent>();

        //app.add_systems(FixedUpdate, (avoid_intersecting,));
        app.add_systems(
            Update,
            (
                spawn_camera_gimbal,
                swap_cameras.run_if(in_state(ActorState::Dead)),
                spectate_entity,
            )
                .in_set(SpectatingSet),
        );
        app.add_systems(
            PostUpdate,
            (
                update_spectatable,
                camera_swivel_and_tilt.run_if(resource_exists::<Spectating>()),
                move_reticle.after(camera_swivel_and_tilt),
                follow_entity.after(TransformSystem::TransformPropagate),
            )
                .in_set(InGameSet::Post),
        );
    }
}

fn update_spectatable(
    mut objects: ResMut<SpectatableObjects>,
    added_spectatables: Query<Entity, Added<Spectatable>>,
    mut removed_spectatables: RemovedComponents<Spectatable>,
) {
    for removing in removed_spectatables.read() {
        if let Some(index) = objects.map.iter().position(|entity| *entity == removing) {
            objects.map.remove(index);
        }
    }
    for entity in &added_spectatables {
        objects.map.push(entity);
    }
}

pub fn camera_swivel_and_tilt(
    mut inner_gimbals: Query<&mut Transform, With<InnerGimbal>>,
    mut outer_gimbals: Query<&mut Transform, (With<OuterGimbal>, Without<InnerGimbal>)>,
    local_input: ResMut<PlayerInput>,
) {
    let Ok(mut outer_transform) = outer_gimbals.get_single_mut() else {
        return;
    };
    let Ok(mut inner_transform) = inner_gimbals.get_single_mut() else {
        return;
    };
    outer_transform.rotation = Quat::from_axis_angle(Vec3::Y, local_input.yaw as f32).into();
    inner_transform.rotation = Quat::from_axis_angle(Vec3::X, local_input.pitch as f32).into();
}

fn swap_cameras(
    spectating: Res<Spectating>,
    mut objects: ResMut<SpectatableObjects>,
    mouse: Res<Input<MouseButton>>,
    mut spectate_events: EventWriter<SpectateEvent>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        objects.current += 1;
    } else if mouse.just_pressed(MouseButton::Right) {
        objects.current -= 1;
    }
    let length = objects.map.len() as isize;
    if length == 0 {
        return;
    }
    let mut index = objects.current % length;
    if index < 0 {
        index += length;
    }
    if index > length {
        index -= length;
    }
    objects.current = index;
    let spectate_new = objects.map.get(index as usize);
    if let Some(spectate_new) = spectate_new {
        if spectating.0 == *spectate_new {
            return;
        }
        spectate_events.send(SpectateEvent {
            entity: *spectate_new,
        })
    }
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

#[derive(Component)]
pub struct FollowEntity(pub Entity);

fn follow_entity(
    mut followers: Query<(&mut Transform, &FollowEntity)>,
    leaders: Query<&GlobalTransform>,
) {
    for (mut transform, follow_entity) in &mut followers {
        let Ok(leader_transform) = leaders.get(follow_entity.0) else {
            continue;
        };
        transform.translation = leader_transform.translation();
    }
}

fn spectate_entity(
    mut spectate_events: EventReader<SpectateEvent>,
    mut gimbal_query: Query<&mut FollowEntity, With<OuterGimbal>>,
    mut spectating: ResMut<Spectating>,
) {
    let Ok(mut follow_entity) = gimbal_query.get_single_mut() else {
        return;
    };
    for new_spectate in spectate_events.read() {
        follow_entity.0 = new_spectate.entity;
        spectating.0 = new_spectate.entity;
    }
}

fn spawn_camera_gimbal(
    mut spectating: ResMut<Spectating>,
    mut commands: Commands,
    mut spectate_events: EventReader<SpectateEvent>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gimbal_query: Query<Entity, With<OuterGimbal>>,
) {
    if let Ok(_) = gimbal_query.get_single() {
        return;
    }
    let Some(spectated) = spectate_events.read().next() else {
        return;
    };

    spectating.0 = spectated.entity;

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 6.5))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::ReinhardLuminance,
                dither: DebandDither::Enabled,
                camera: Camera {
                    order: 1, // overwite the spectator cam
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            Fxaa::default(),
            PlayerCam,
            BloomSettings::default(),
            Name::new("Player Camera"),
            AvoidIntersecting {
                dir: Vec3::Z,
                max_toi: 6.5,
                buffer: 0.05,
            },
            /*
            FogSettings {
                color: Color::rgba(0.05, 0.1, 0.4, 1.0),
                falloff: FogFalloff::from_visibility_colors(
                    10000.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
                ..default()
            },*/
        ))
        .id();

    let outer_gimbal = commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
            }),
            OuterGimbal,
            FollowEntity(spectated.entity),
            Name::new("Outer Gimbal"),
        ))
        .id();

    let inner_gimbal = commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(0., 1., 0.),
                ..default()
            }),
            InnerGimbal,
            Name::new("Inner Gimbal"),
        ))
        .id();

    let reticle = commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
            }),
            Reticle {
                max_distance: 7.0,
                cam_height: 1.0,
            },
            Name::new("Reticle"),
        ))
        .id();

    let mut material = StandardMaterial::default();
    material.base_color = Color::hex("800000").unwrap().into();
    material.perceptual_roughness = 0.97;
    material.reflectance = 0.0;
    let red = materials.add(material);
    let ret_mesh = commands
        .spawn(PbrBundle {
            material: red.clone(),
            mesh: _meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 })),
            ..default()
        })
        .id();

    commands.entity(reticle).push_children(&[ret_mesh]);
    commands.entity(inner_gimbal).push_children(&[camera]);
    commands
        .entity(outer_gimbal)
        .push_children(&[inner_gimbal, reticle]);
}

/*
pub fn avoid_intersecting(
    rapier_context: Res<RapierContext>,
    all_global_transforms: Query<&GlobalTransform>,
    mut avoiders: Query<(&mut Transform, &Parent, &AvoidIntersecting)>,
) {
    let filter = QueryFilter::only_fixed()
        .exclude_sensors()
        .groups(CollisionFilters);
    for (mut transform, parent, avoid) in &mut avoiders {
        let adjusted_transform = if let Ok(global) = all_global_transforms.get(parent.get()) {
            global.compute_transform()
        } else {
            Transform::default()
        };
        let (toi, normal) = if let Some((_entity, intersection)) = rapier_context
            .cast_ray_and_get_normal(
                adjusted_transform.translation,
                adjusted_transform.rotation * avoid.dir,
                avoid.max_toi + avoid.buffer,
                true,
                filter,
            ) {
            (intersection.toi, intersection.normal)
        } else {
            (avoid.max_toi + avoid.buffer, Vec3::ZERO)
        };
        transform.translation = avoid.dir * toi + (normal * avoid.buffer);
    }
}
*/

#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub cam_height: f32,
}

#[derive(Component)]
pub struct SpectatorCam;

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Spectating(pub Entity);

#[derive(Debug)]
pub struct Possessable {
    pub entity: Entity,
    pub active: bool,
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Possessing(pub Possessable);

#[derive(Resource, Default, Clone, Debug)]
pub struct SpectatableObjects {
    map: Vec<Entity>,
    current: isize,
}

#[derive(Component, Debug)]
pub struct Spectatable;

#[derive(Component, Debug)]
pub struct PlayerCam;

#[derive(Component, Clone, Debug)]
pub struct AvoidIntersecting {
    pub dir: Vec3,
    pub max_toi: f32,
    pub buffer: f32,
}

#[derive(Component, Debug)]
pub struct OuterGimbal;
#[derive(Component, Debug)]
pub struct InnerGimbal;

#[derive(Event)]
pub struct SpectateEvent {
    pub entity: Entity,
}
#[derive(Event)]
pub struct PossessEvent {
    pub entity: Entity,
}
