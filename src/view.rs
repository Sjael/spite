use bevy::{prelude::*, render::camera::RenderTarget, core_pipeline::{bloom::BloomSettings, tonemapping::{DebandDither, Tonemapping}, fxaa::Fxaa}, transform::TransformSystem};
use bevy_rapier3d::prelude::{RapierContext, QueryFilter, RapierTransformPropagateSet, PhysicsSet};
use leafwing_input_manager::prelude::ActionState;

use crate::{player::{Player, PlayerInput, NetworkOwner, Reticle, player_movement}, game_manager::{CharacterState, TERRAIN_GROUPING, GROUND_GROUPING, CAMERA_GROUPING}, ability::Ability, input::Slot, GameState};


#[derive(Component)]
pub struct SpectatorCam;

#[derive(Resource)]
pub struct Spectating(pub Option<Entity>);

#[derive(Resource, Default, Clone, Debug)]
pub struct SpectatableObjects{
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
pub struct TransformGimbal;
#[derive(Component, Debug)]
pub struct OuterGimbal;
#[derive(Component, Debug)]
pub struct InnerGimbal;

pub struct PossessEvent{
    pub entity: Entity,
}

pub struct ViewPlugin;
impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Spectating(None))
            .insert_resource(SpectatableObjects::default());

        app.add_event::<PossessEvent>();

        app.add_system(
            avoid_intersecting.in_schedule(CoreSchedule::FixedUpdate).in_set(OnUpdate(GameState::InGame))
        );
        app.add_systems((
            spawn_camera_gimbal,
            swap_cameras,
            possess_entity,
        ).in_set(OnUpdate(GameState::InGame)));
        app.add_systems((
            update_spectatable.run_if(in_state(GameState::InGame)),
            camera_swivel_and_tilt.run_if(in_state(GameState::InGame)),
        ).in_base_set(CoreSet::PostUpdate));
    }
}

fn update_spectatable(
    mut objects: ResMut<SpectatableObjects>,
    added_spectatables: Query<Entity, Added<Spectatable>>,
    mut removed_spectatables: RemovedComponents<Spectatable>,
){
    for removing in removed_spectatables.iter(){
        if let Some(index) = objects.map.iter().position(|entity| *entity == removing){
            objects.map.remove(index);
            dbg!(objects.clone());
        }
    }
    for entity in added_spectatables.iter(){
        objects.map.push(entity);
        dbg!(objects.clone());
    }
}


pub fn camera_swivel_and_tilt(
    mut inner_gimbals: Query<&mut Transform, (With<InnerGimbal>, Without<Player>)>,
    mut outer_gimbals: Query<&mut Transform, (With<OuterGimbal>, Without<Player>, Without<InnerGimbal>)>,
    players: Query<Option<&NetworkOwner>, With<Spectatable>>,
    local_input: ResMut<PlayerInput>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    let Ok(mut outer_transform) = outer_gimbals.get_single_mut() else {return};
    let Ok(mut inner_transform) = inner_gimbals.get_single_mut() else {return};
    if let Ok(owner) = players.get(spectating) {      
        if owner.is_none(){
            outer_transform.rotation = Quat::from_axis_angle(Vec3::Y, local_input.yaw as f32).into();
        }
        inner_transform.rotation = Quat::from_axis_angle(Vec3::X, local_input.pitch as f32).into();
    }      
}

fn swap_cameras(
    spectating: Res<Spectating>,
    mut objects: ResMut<SpectatableObjects>,
    //mut spawn_events: EventReader<SpawnEvent>,
    //query: Query<Entity, Added<Player>>,
    mouse: Res<Input<MouseButton>>,
    character_alive: Res<State<CharacterState>>,
    mut possess_events: EventWriter<PossessEvent>,
){
    if character_alive.0 == CharacterState::Dead{
        let Some(spectating_now) = spectating.0 else { return };
        if mouse.just_pressed(MouseButton::Left){
            objects.current += 1;
        } else if mouse.just_pressed(MouseButton::Right){
            objects.current -= 1;
        }
        let length = objects.map.len();
        if length == 0 { 
            return 
        }
        let mut index = objects.current % length as isize;
        if index < 0{
            index += length as isize;
        }
        let spectate_new = objects.map.get(index as usize);
        if let Some(spectate_new) = spectate_new{
            if spectating_now == *spectate_new{
                return
            }
            possess_events.send(PossessEvent{
                entity: *spectate_new,
            })
        }
    }
}


fn possess_entity(
    mut possess_events: EventReader<PossessEvent>,
    mut gimbal_query: Query<Entity, With<TransformGimbal>>,
    mut y_axis_gimbal: Query<&mut Transform, With<OuterGimbal>>,
    mut commands: Commands,
    mut spectating: ResMut<Spectating>,
){    
    let Ok(entity) = gimbal_query.get_single_mut() else { return };
    let Ok(mut transform) = y_axis_gimbal.get_single_mut() else { return };
    for possessed in possess_events.iter(){
        transform.rotation = Quat::IDENTITY;
        spectating.0 = Some(possessed.entity);
        commands.entity(entity).set_parent(possessed.entity.clone());
        dbg!(possessed.entity);
    }
}


/*

fn toggle_cam(
    mut editor_events: EventReader<PossessEvent>,
    //mut prev_active_cams: ResMut<PreviouslyActiveCameras>,
    mut cam_query: Query<(Entity, &mut Camera)>,
) {

    for event in editor_events.iter() {
        if let EditorEvent::Toggle { now_active } = *event {
            if now_active {
                // Add all currently active cameras
                for (e, mut cam) in cam_query
                    .iter_mut()
                    //  Ignore non-Window render targets
                    .filter(|(_e, c)| matches!(c.target, RenderTarget::Window(_)))
                    .filter(|(_e, c)| c.is_active)
                {
                    prev_active_cams.0.insert(e);
                    cam.is_active = false;
                }
            } else {
                for cam in prev_active_cams.0.iter() {
                    if let Ok((_e, mut camera)) = cam_query.get_mut(*cam) {
                        camera.is_active = true;
                    }
                }
                prev_active_cams.0.clear();
            }
        }
    }
}
 */


 fn spawn_camera_gimbal(
    mut spectating: ResMut<Spectating>,
    mut commands: Commands,
    mut possess_events: EventReader<PossessEvent>,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gimbal_query: Query<Entity, With<TransformGimbal>>,
){
    if let Ok(_) = gimbal_query.get_single(){
        return;
    }
    let Some(possessed) = possess_events.iter().next() else { return };
    spectating.0 = Some(possessed.entity);

    let camera = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 6.5))
                .looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::ReinhardLuminance,
            dither: DebandDither::Enabled,
            camera: Camera{
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
    )).id();


    let transform_gimbal = commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
        }),
        TransformGimbal,
        Name::new("Transform Gimbal"),
    )).id();
    let outer_gimbal = commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
        }),
        OuterGimbal,
        Name::new("Outer Gimbal"),
    )).id();

    let inner_gimbal = commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(0., 1., 0.),
                ..default()
        }),
        InnerGimbal,
        Name::new("Inner Gimbal"),
    )).id();
    
    let reticle = commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
        }),
        Reticle {
            max_distance: 7.0,
            from_height: 4.0,
        },
        Name::new("Reticle"),
    )).id();      
    
    let mut material = StandardMaterial::default();
    material.base_color = Color::hex("800000").unwrap().into();
    material.perceptual_roughness = 0.97;
    material.reflectance = 0.0;
    let red = materials.add(material);
    let ret_mesh = commands.spawn(PbrBundle {
        material: red.clone(),
        mesh: _meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 })),
        ..default()
    }).id();

    commands.entity(inner_gimbal).push_children(&[camera]);
    commands.entity(outer_gimbal).push_children(&[inner_gimbal]);
    commands.entity(transform_gimbal).push_children(&[outer_gimbal]);
    commands.entity(reticle).push_children(&[ret_mesh]);
    
    commands.entity(possessed.entity).push_children(&[transform_gimbal, reticle]);
}

pub fn avoid_intersecting(
    rapier_context: Res<RapierContext>,
    global_query: Query<&GlobalTransform>,
    mut avoid: Query<(&mut Transform, &Parent, &AvoidIntersecting)>,
) {
    let filter = QueryFilter::only_fixed().exclude_sensors().groups(CAMERA_GROUPING);
    for (mut transform, parent, avoid) in &mut avoid {
        let adjusted_transform = if let Ok(global) = global_query.get(parent.get()) {
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