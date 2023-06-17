use std::io::Cursor;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*, core_pipeline::{fxaa::Fxaa, tonemapping::{Tonemapping, DebandDither}}, winit::WinitWindows, window::PrimaryWindow
};
use winit::window::Icon;
use bevy_fly_camera::{camera_movement_system, mouse_motion_system, FlyCamera};
use bevy_rapier3d::prelude::*;
use sacred_aurora::{
    game_manager::{Fountain, CharacterState, Team, PLAYER_GROUPING, GROUND_GROUPING, TERRAIN_GROUPING},  
    stats::*, 
    assets::*, 
    GameState, 
    ability::{EffectApplyType, TargetsInArea, TargetsToEffect, Tags, TagInfo, TagType, ScanEffect, OnEnterEffect, Ticks, LastHitTimers, EffectApplyTargets, TargetSelection
}, view::Spectatable};

fn main() {
    //std::env::set_var("RUST_BACKTRACE", "1");
    let mut app = App::new();
    sacred_aurora::app_plugins_both(&mut app);
    // Systems
    app.add_system(setup_map.in_schedule(OnEnter(GameState::InGame)));
    app.add_system(setup_camera.on_startup());
    app.add_systems((
        camera_movement_system, 
        mouse_motion_system
        )
        .in_set(OnUpdate(CharacterState::Dead))
        .in_set(OnUpdate(GameState::InGame))
    );
    app.add_system(set_window_icon.on_startup());
    app.run();
}

pub fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    icons: Res<Icons>,
    models: Res<Models>,
    scenes: Res<Scenes>,
) {

    //ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 4.0,
                subdivisions: 5,
            })),
            material: materials.add(icons.basic_attack.clone().into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        GROUND_GROUPING,
        Collider::cuboid(50.0, 0.1, 50.0),
        Name::new("Plane"),
    ));

    //moving platform
    commands.spawn((
        GlobalTransform::default(),
        Transform::default(),
        meshes.add(Mesh::from(shape::Plane {
            size: 10.0,
            subdivisions: 2,
        })),
        //Loader::<StandardMaterial>::new("icons/frostbolt.png"),
        Visibility::default(),
        ComputedVisibility::default(),
        //RigidBody::KinematicVelocityBased,
        //Collider::cuboid(5.0, 0.1, 5.0),
        /*
        Friction {
            coefficient: 0.0,
            ..default() //combine_rule: CoefficientCombineRule::Max,
        },
        */
        Name::new("Platform"),
    ));

    let tower = commands.spawn((        
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(-3.0, 0.5, -22.0),
                ..default()
        }),
        meshes.add(shape::Capsule{
            radius: 0.7,
            depth: 2.0,
            ..default()
        }.into()),
        materials.add(StandardMaterial::from(Color::RED)),
        
        Collider::capsule(Vec3::ZERO, Vec3::Y, 0.7),
        RigidBody::Fixed,
        TERRAIN_GROUPING,
        Team(5),
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
        Attribute::<Health>::new(4000.0),
        Attribute::<Min<Health>>::new(0.0),
        Attribute::<Max<Health>>::new(10000.0),
        Attribute::<Regen<Health>>::new(0.0),
        Name::new("Tower"),       
        Spectatable, 
    )).id();

    let tower_range = commands.spawn((
        SpatialBundle::default(),
        Collider::cylinder(1.0, 7.),
        Sensor,
        EffectApplyTargets{
            number_of_targets: 1,
            target_selection: TargetSelection::Closest,
        },
        EffectApplyType::Scan(ScanEffect{
            ticks: Ticks::Unlimited{
                interval: 2000,
            },
            ..default()
        }),
        TargetsInArea::default(),
        TargetsToEffect::default(),
        Tags{
            list: vec![
                TagInfo{
                    tag: TagType::Damage(25.0),
                    team:4,
                }
            ]
        },
        Name::new("Range Collider"),      
    )).id();
    
    commands.entity(tower).push_children(&[tower_range]);

    // target dummy
    commands.spawn((        
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(3.0, 0.5, -12.0),
                ..default()
        }),
        meshes.add(shape::Capsule{
            radius: 0.4,
            ..default()
        }.into()),
        materials.add(StandardMaterial::from(Color::INDIGO)),
        
        Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
        RigidBody::Fixed,
        PLAYER_GROUPING,
        Team(5),
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
        Attribute::<Health>::new(4000.0),
        Attribute::<Min<Health>>::new(0.0),
        Attribute::<Max<Health>>::new(10000.0),
        Attribute::<Regen<Health>>::new(0.0),
        Name::new("Target Dummy"),       
        Spectatable, 
    ));

    // Scanning Damage zone
    commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(-10.0, 0.0, 10.0),
                ..default()
        }),
        meshes.add(Mesh::from(shape::Plane {
            size: 4.0,
            subdivisions: 2,
        })),
        materials.add(StandardMaterial::from(Color::MAROON)),
        Collider::cuboid(2.0, 0.3, 2.0),
        Sensor,
        EffectApplyType::Scan(ScanEffect::default()),
        TargetsInArea::default(),
        TargetsToEffect::default(),
        Tags{
            list: vec![
                TagInfo{
                    tag: TagType::Damage(12.0),
                    team:4,
                }
            ]
        },
        Name::new("DamageFountain"),
    ));

    // Damage zone
    commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(10.0, 0.0, 10.0),
                ..default()
        }),
        meshes.add(Mesh::from(shape::Plane {
            size: 4.0,
            subdivisions: 2,
        })),
        materials.add(StandardMaterial::from(Color::GOLD)),
        Collider::cuboid(2.0, 0.3, 2.0),
        Sensor,
        Tags{
            list: vec![
                TagInfo{
                    tag: TagType::Damage(44.0),
                    team:4,
                }
            ]
        },
        EffectApplyType::OnEnter(OnEnterEffect{
            target_penetration: 2,
            ticks: Ticks::Unlimited { interval: 500 },
            hittimers: LastHitTimers::default(),
        }),
        TargetsInArea::default(),
        TargetsToEffect::default(),
        Spectatable, 
        Name::new("DamageFountain2"),
    ));

    // Fountain 
    commands.spawn((
        SpatialBundle::from_transform(
            Transform {
                translation: Vec3::new(10.0, 0.0, -10.0),
                ..default()
        }),
        meshes.add(Mesh::from(shape::Plane {
            size: 4.0,
            subdivisions: 2,
        })),
        materials.add(StandardMaterial::from(Color::GREEN)),
        Collider::cuboid(2.0, 0.3, 2.0),
        Sensor,
        Fountain,
        EffectApplyType::Scan(ScanEffect{
            ticks: Ticks::Unlimited{
                interval: 2000,
            },
            ..default()
        }),
        Tags{
            list: vec![
                TagInfo{
                    tag: TagType::Heal(28.0),
                    team:1,
                },
                TagInfo{
                    tag: TagType::Damage(44.0),
                    team:1,
                }
            ]
        },
        TargetsInArea::default(),
        TargetsToEffect::default(),
        Spectatable, 
        Name::new("Healing Fountain"),
    ));

    // sky 
    let _sky = commands
    .spawn((
        SceneBundle {
            scene: models.skybox.clone(),
            transform: Transform {
                //scale: Vec3::splat(3.0),
                ..default()
            },
            // make unlit might make fog work? need to apply standardmaterial
            //unlit: true,
            ..default()
        },
        Name::new("Sky"),
    ))
    .insert((NotShadowCaster, NotShadowReceiver))
    .id();

    //lighting
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 50000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                ..default()
            },
            ..default()
        },
        Name::new("Sun"),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.25,
    });
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..default()
        },
        Name::new("PointLight"),
    ));

    // arena
    commands
        .spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(0., -0.5, 0.),
                    scale: Vec3::new(4., 4., 4.),
                    ..default()
                },
                ..default()
            },
            Name::new("Arena"),
        ))
        .with_children(|commands| {
            commands.spawn(SceneBundle {
                scene: scenes.arena_map.clone(),
                ..default()
            });
        });
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(11., 5., 24.))
                .looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::ReinhardLuminance,
            dither: DebandDither::Enabled,
            ..default()
        },
        Fxaa::default(),
        Name::new("Spectator Camera"),
        FlyCamera{
            sensitivity: 12.0,
            ..default()
        },
    ));
}


fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../assets/icons/fireball.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}