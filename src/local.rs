use std::io::Cursor;

use bevy::{
    core_pipeline::{
        fxaa::Fxaa,
        tonemapping::{DebandDither, Tonemapping},
    },
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    window::PrimaryWindow,
    winit::WinitWindows,
};
use bevy_fly_camera::{camera_movement_system, mouse_motion_system};
use bevy_rapier3d::prelude::*;
use sacred_aurora::{
    ability::{
        bundles::Caster, Ability, FilteredTargets, FiringInterval, PausesWhenEmpty, TagInfo, Tags,
        TargetFilter, TargetSelection, TargetsHittable, TargetsInArea, TickBehavior, Ticks,
    },
    actor::{
        buff::{BuffInfo, BuffMap, BuffTargets, BuffType},
        crowd_control::{CCInfo, CCMap, CCType},
        stats::*,
        view::Spectatable,
        HasHealthBar, IncomingDamageLog, Tower,
    },
    area::non_damaging::ObjectiveHealthOwner,
    assets::*,
    game_manager::{
        CharacterState, Fountain, InGameSet, GROUND_GROUPING, PLAYER_GROUPING, TEAM_1, TEAM_2,
        TEAM_NEUTRAL, TERRAIN_GROUPING,
    },
    GameState,
};
use winit::window::Icon;

fn main() {
    let mut app = App::new();
    sacred_aurora::app_plugins_both(&mut app);
    // Systems
    app.add_systems(OnEnter(GameState::InGame), setup_map);
    app.add_systems(Startup, set_window_icon);
    app.add_systems(Startup, setup_camera);
    app.add_systems(
        Update,
        (camera_movement_system, mouse_motion_system)
            .in_set(InGameSet::Update)
            .run_if(in_state(CharacterState::Dead)),
    );
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

    let tower = commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(-3.0, 0.5, -22.0),
                ..default()
            }),
            meshes.add(
                shape::Capsule {
                    radius: 0.7,
                    depth: 2.0,
                    ..default()
                }
                .into(),
            ),
            materials.add(StandardMaterial::from(Color::RED)),
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.7),
            RigidBody::Fixed,
            TERRAIN_GROUPING,
            TEAM_NEUTRAL,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
            {
                let mut attributes = Attributes::default();
                *attributes.entry(Stat::Health.into()).or_default() = 33.0;
                *attributes
                    .entry(Stat::MagicalProtection.into())
                    .or_default() = 60.0;
                *attributes
                    .entry(Stat::PhysicalProtection.into())
                    .or_default() = 60.0;
                attributes
            },
            Tower,
            Name::new("Tower"),
            Spectatable,
            ObjectiveHealthOwner {
                not_looking_range: 13.0,
                looking_range: 20.0,
            },
        ))
        .id();

    let tower_range = commands
        .spawn((
            SpatialBundle::default(),
            TEAM_NEUTRAL,
            Name::new("Range Collider"),
        ))
        .insert((
            Collider::cylinder(1.0, 7.),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
            Sensor,
            TargetsInArea::default(),
            FiringInterval(2.0),
            Ticks::Unlimited,
            TickBehavior::static_timer(),
            PausesWhenEmpty,
            Caster(tower),
            TargetFilter {
                number_of_targets: 1,
                target_selection: TargetSelection::Closest,
            },
            FilteredTargets::default(),
            TargetsHittable::default(),
            Tags {
                list: vec![TagInfo::Homing(Ability::Fireball)],
            },
        ))
        .id();

    commands.entity(tower).push_children(&[tower_range]);

    // target dummy
    commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(-3.0, 0.5, -17.0),
                ..default()
            }),
            meshes.add(
                shape::Capsule {
                    radius: 0.4,
                    ..default()
                }
                .into(),
            ),
            materials.add(StandardMaterial::from(Color::INDIGO)),
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
            RigidBody::Dynamic,
            Friction::coefficient(2.0),
            LockedAxes::ROTATION_LOCKED,
            PLAYER_GROUPING,
            TEAM_2,
            CCMap::default(),
            BuffMap::default(),
            HasHealthBar,
            IncomingDamageLog::default(),
            Spectatable,
            Name::new("Target Dummy"),
        ))
        .insert((Attributes::default(),));

    // Scanning Damage zone
    commands.spawn((
        SpatialBundle::from_transform(Transform {
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
        FiringInterval(5.0),
        Ticks::Unlimited,
        TickBehavior::static_timer(),
        TargetsInArea::default(),
        TargetsHittable::default(),
        TEAM_NEUTRAL,
        Tags {
            list: vec![
                TagInfo::Damage(12.0),
                TagInfo::CC(CCInfo {
                    cctype: CCType::Stun,
                    duration: 3.0,
                }),
            ],
        },
        Name::new("DamageFountain"),
    ));

    // Damage zone
    commands.spawn((
        SpatialBundle::from_transform(Transform {
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
        Tags {
            list: vec![
                TagInfo::Damage(27.0),
                TagInfo::Buff(BuffInfo {
                    stat: AttributeTag::Modifier {
                        modifier: Modifier::Mul,
                        target: Box::new(Stat::Speed.into()),
                    },
                    amount: 20.0,
                    max_stacks: 3,
                    duration: 10.0,
                    ..default()
                }),
                TagInfo::CC(CCInfo {
                    cctype: CCType::Silence,
                    duration: 7.0,
                }),
            ],
        },
        FiringInterval(1.0),
        TickBehavior::individual(),
        Ticks::Unlimited,
        TEAM_NEUTRAL,
        TargetsInArea::default(),
        TargetsHittable::default(),
        Spectatable,
        Name::new("DamageFountain2"),
    ));

    // Fountain
    commands.spawn((
        SpatialBundle::from_transform(Transform {
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
        TEAM_1,
        TargetsInArea::default(),
        TickBehavior::individual(),
        Ticks::Unlimited,
        FiringInterval(1.0),
        Tags {
            list: vec![
                TagInfo::Heal(28.0),
                TagInfo::Damage(44.0),
                TagInfo::Buff(BuffInfo {
                    stat: AttributeTag::Modifier {
                        modifier: Modifier::Add,
                        target: Box::new(Stat::PhysicalPenetration.into()),
                    },
                    amount: 5.0,
                    max_stacks: 6,
                    duration: 18.0,
                    bufftargets: BuffTargets::Allies,
                    bufftype: BuffType::Buff,
                    ..default()
                }),
            ],
        },
        TargetsHittable::default(),
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
        /*
        FlyCamera {
            sensitivity: 12.0,
            ..default()
        },
        */
    ));
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(primary_entity) = primary_window.get_single() else { return };
    let Some(primary) = windows.get_window(primary_entity) else { return };
    let icon_buf = Cursor::new(include_bytes!("../assets/icons/fireball.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
