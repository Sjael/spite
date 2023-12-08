use bevy::core_pipeline::{
    bloom::BloomSettings,
    fxaa::Fxaa,
    tonemapping::{DebandDither, Tonemapping},
};

use crate::{
    actor::player::{input::PlayerInput, reticle::Reticle, LocalPlayer},
    prelude::*,
};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Focus>();

        app.add_systems(
            PostUpdate,
            (camera_swivel_and_tilt, spectate_entity, follow_entity).in_set(CameraSet),
        );

        app.add_systems(
            PostUpdate,
            (spectate_entity, focus_entity, follow_entity)
                .chain()
                .in_set(FocusSet),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CameraSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusSet;

/// Current target focus for this camera.
#[derive(Component, Debug, Clone, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[reflect(Component)]
pub struct Focus(pub Entity);

impl FromWorld for Focus {
    fn from_world(_world: &mut World) -> Self {
        Focus(Entity::PLACEHOLDER)
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

/// Propagate the spectating entity to the current camera focus.
pub fn spectate_entity(
    mut spectatables: Query<Entity, With<Spectatable>>,
    mouse: Res<Input<MouseButton>>,
    mut spectators: Query<(&mut Focus, &Spectating), Changed<Spectating>>,
) {
    for (mut focus, spectating) in &mut spectators {
        if let Some(entity) = spectating.spectating() {
            if focus.0 != entity {
                focus.0 = entity;
            }
        }
    }
}

/// Follow the translation of another `Entity`.
#[derive(Component)]
pub struct FollowTranslation(pub Entity);

pub fn follow_entity(
    mut followers: Query<(&mut Transform, &FollowTranslation), Without<Parent>>,
    leaders: Query<&GlobalTransform>,
) {
    for (mut transform, follow) in &mut followers {
        let Ok(leader_transform) = leaders.get(follow.0) else {
            continue;
        };
        transform.translation = leader_transform.translation();
    }
}

/// Sync the focus with the entity to follow.
pub fn focus_entity(mut focuses: Query<(&mut FollowTranslation, &Focus), Changed<Focus>>) {
    for (mut follow, focus) in &mut focuses {
        follow.0 = focus.0;
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player: Res<LocalPlayer>,
) {
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
            FollowTranslation(local_player.0),
            Focus(local_player.0),
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
            mesh: meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 })),
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

#[derive(Component)]
pub struct SpectatorCam;

#[derive(Debug)]
pub struct Possessable {
    pub entity: Entity,
    pub active: bool,
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Possessing(pub Possessable);

#[derive(Component, Default, Clone, Debug)]
pub struct Spectating {
    available: Vec<Entity>,
    index: usize,
}

impl Spectating {
    /// Set the list of available entities to spectate.
    pub fn set_available(&mut self, available: Vec<Entity>) {
        self.available = available;
    }

    /// List of available entities to spectate.
    pub fn available(&self) -> &[Entity] {
        self.available.as_slice()
    }

    /// Current entity we are spectating.
    pub fn spectating(&self) -> Option<Entity> {
        self.available.get(self.index).copied()
    }

    /// Set the index of the currently spectated entity.
    pub fn set_current(&mut self, index: usize) {
        self.index = index;
    }

    /// Cycle up the index once.
    pub fn cycle_up(&mut self) {
        self.index += 1;
        self.index = self.index % self.available.len();
    }

    /// Cycle down the index once.
    pub fn cycle_down(&mut self) {
        if self.index == 0 {
            self.index = self.available.len() - 1;
        } else {
            self.index -= 1;
        }
    }
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
