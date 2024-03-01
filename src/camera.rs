use bevy::core_pipeline::{
    fxaa::Fxaa,
    tonemapping::{DebandDither, Tonemapping},
};
use bevy_fly_camera::FlyCamera;

use crate::{
    actor::player::{input::PlayerInput, LocalPlayer, LocalPlayerId, Player},
    prelude::*,
};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FollowTranslation>();
        app.register_type::<Spectating>();
        app.register_type::<MainCam>();

        // app.add_systems(Update, (change_target_cam, update_main_cam));
        app.add_systems(
            PostUpdate,
            (
                (propagate_follow, follow_entity, camera_swivel_and_tilt).chain(),
                populate_spectatable,
                update_spectatable,
                spectate_owned.run_if(resource_exists::<LocalPlayer>),
                rotate_spectating,
                move_reticle,
            )
                .in_set(CameraSet),
        );

        app.add_systems(FixedUpdate, (spawn_camera,).in_set(InGameSet::Pre));
    }
}

#[derive(Resource, Deref, Reflect)]
pub struct MainCam(Entity);

fn update_main_cam(
    mut commands: Commands,
    main_cam: Option<Res<MainCam>>,
    query: Query<(&Camera, Entity, Has<PlayerCam>)>,
    added: Query<(), Added<Camera>>,
){
    if let Some(main_cam) = main_cam {
        if let Ok((_, _, is_player_cam)) = query.get(**main_cam) {
            if is_player_cam{
                println!("playercam already exists");
                return;
            }
        }
    }
    for (_, entity, _) in query.iter(){
        let Ok(_) = added.get(entity) else { continue };
        println!("added maincam");
        commands.insert_resource(MainCam(entity));
    }
}

fn change_target_cam(
    main_cam: Option<Res<MainCam>>,
    query: Query<Entity, (With<Node>, Without<Parent>)>,
    mut commands: Commands,
) {
    let Some(main_cam) = main_cam else {return};
    if !main_cam.is_changed(){
        return
    }
    for root_ent in query.iter(){
        commands.entity(root_ent).insert(TargetCamera(**main_cam));
        println!("set targets");
    }
}

/// Propagate the spectating entity to the current camera focus.
fn propagate_follow(mut spectators: Query<(&mut FollowTranslation, &Spectating), Changed<Spectating>>) {
    for (mut follow, spectating) in &mut spectators {
        if let Some(entity) = spectating.get() {
            if follow.0 == entity {
                continue
            }
            follow.0 = entity;
        }
    }
}

fn follow_entity(mut followers: Query<(&mut Transform, &FollowTranslation)>, leaders: Query<&GlobalTransform>) {
    for (mut transform, follow) in &mut followers {
        let Ok(leader_transform) = leaders.get(follow.0) else { continue };
        transform.translation = leader_transform.translation();
    }
}

fn camera_swivel_and_tilt(
    mut inner_gimbals: Query<&mut Transform, With<InnerGimbal>>,
    mut outer_gimbals: Query<&mut Transform, (With<OuterGimbal>, Without<InnerGimbal>)>,
    local_input: ResMut<PlayerInput>,
) {
    let Ok(mut outer_transform) = outer_gimbals.get_single_mut() else { return };
    let Ok(mut inner_transform) = inner_gimbals.get_single_mut() else { return };
    outer_transform.rotation = Quat::from_axis_angle(Vec3::Y, local_input.yaw as f32).into();
    inner_transform.rotation = Quat::from_axis_angle(Vec3::X, local_input.pitch as f32).into();
}

fn populate_spectatable(
    previous: Query<Entity, With<Spectatable>>,
    mut cameras: Query<&mut Spectating, Added<Spectating>>,
) {
    for mut spectating in cameras.iter_mut() {
        spectating.set_available(previous.iter().collect());
    }
}

fn update_spectatable(
    mut cameras: Query<&mut Spectating>,
    added: Query<(Entity, Option<&Team>), Added<Spectatable>>, // 'team spectate only' later
    mut removed: RemovedComponents<Spectatable>,
) {
    for (added_entity, _) in added.iter() {
        for mut spectating in cameras.iter_mut() {
            spectating.push(added_entity);
        }
    }
    for entity in removed.read() {
        for mut spectating in cameras.iter_mut() {
            spectating.remove(entity);
            spectating.cycle_down();
        }
    }
}

fn spectate_owned(mut cameras: Query<&mut Spectating, Changed<Spectating>>, local_player: Res<LocalPlayer>) {
    for mut spectating in cameras.iter_mut() {
        let available = spectating.available().to_vec();
        if let Some(index) = available.iter().position(|&i| i == **local_player) {
            spectating.set_current(index);
        }
    }
}

fn rotate_spectating(
    mut cameras: Query<(&mut Spectating, &PlayerBoom)>,
    local_player: Option<Res<LocalPlayer>>,
    local_player_id: Option<Res<LocalPlayerId>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if let Some(_) = local_player {
        return
    }
    let Some(local_player_id) = local_player_id else { return };
    for (mut spectating, player) in cameras.iter_mut() {
        if **player != *local_player_id {
            continue
        }
        if mouse.just_pressed(MouseButton::Left) {
            spectating.cycle_up();
        } else if mouse.just_pressed(MouseButton::Right) {
            spectating.cycle_down();
        }
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player: Option<Res<LocalPlayer>>,
    local_player_id: Option<Res<LocalPlayerId>>,
    player_cams: Query<(), With<PlayerCam>>,
) {
    let Some(local_player) = local_player else { return };
    let Some(local_player_id) = local_player_id else { return };
    if player_cams.iter().next().is_some() {
        return
    }

    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 6.5)).looking_at(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::ReinhardLuminance,
                dither: DebandDither::Enabled,
                camera: Camera {
                    order: 1, // overwite the spectator cam
                    ..default()
                },
                ..default()
            },
            Fxaa::default(),
            PlayerCam,
            Name::new("Player Camera"),
            // AvoidIntersecting {
            //     dir: Vec3::Z,
            //     max_toi: 6.5,
            //     buffer: 0.05,
            // },
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
            PlayerBoom(**local_player_id), // who owns this boom, assigned when connected
            Spectating::new(**local_player),
            FollowTranslation(**local_player),
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
            mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.2))),
            ..default()
        })
        .id();
    commands.entity(reticle).push_children(&[ret_mesh]);
    commands.entity(inner_gimbal).push_children(&[camera]);
    commands.entity(outer_gimbal).push_children(&[inner_gimbal, reticle]);
}

pub fn spawn_spectator_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(11., 5., 24.)).looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::ReinhardLuminance,
            dither: DebandDither::Enabled,
            camera: Camera {
                is_active: false, 
                ..default()
            },
            ..default()
        },
        FlyCamera {
            sensitivity: 12.0,
            ..default()
        },
        Fxaa::default(),
        Name::new("Spectator Camera"),
    ));
}

fn move_reticle(mut reticles: Query<(&mut Transform, &Reticle)>, player_input: Res<PlayerInput>) {
    for (mut transform, reticle) in &mut reticles {
        let current_angle = player_input.pitch.clamp(-1.57, 0.);
        transform.translation.z = (1.57 + current_angle).tan() * -reticle.cam_height;
        transform.translation.z = transform.translation.z.clamp(-reticle.max_distance, 0.);
    }
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CameraSet;

/// Current target focus for this camera.
#[derive(Component, Debug, Clone, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[reflect(Component)]
pub struct FollowTranslation(pub Entity);

impl FromWorld for FollowTranslation {
    fn from_world(_world: &mut World) -> Self {
        FollowTranslation(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Spectating {
    available: Vec<Entity>,
    index: usize,
}

impl Spectating {
    /// New spectating and starting entity
    pub fn new(start: Entity) -> Spectating {
        Self {
            available: vec![start],
            index: 0,
        }
    }
    /// Set the list of available entities to spectate.
    pub fn set_available(&mut self, available: Vec<Entity>) {
        self.available = available;
    }
    pub fn push(&mut self, added: Entity) {
        self.available.push(added);
    }
    pub fn remove(&mut self, removed: Entity) {
        self.available.retain(|e| e != &removed);
    }

    /// List of available entities to spectate.
    pub fn available(&self) -> &[Entity] {
        self.available.as_slice()
    }

    /// Current entity we are spectating.
    pub fn get(&self) -> Option<Entity> {
        self.available.get(self.index).copied()
    }

    /// Get the index of the currently spectated entity.
    pub fn current(&self) -> usize {
        self.index
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
#[derive(Component, Debug, Deref)]
pub struct PlayerBoom(Player);

// #[derive(Component, Clone, Debug)]
// pub struct AvoidIntersecting {
//     pub dir: Vec3,
//     pub max_toi: f32,
//     pub buffer: f32,
// }

#[derive(Component, Debug)]
pub struct OuterGimbal;
#[derive(Component, Debug)]
pub struct InnerGimbal;
