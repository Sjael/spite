use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use crate::{
    ability::{
        buff::{BuffInfo, BuffMap, BuffTargets, BuffType},
        cast::Caster,
        cast::Tower,
        crowd_control::{CCInfo, CCMap, CCType},
        Ability, FilteredTargets, FiringInterval, PausesWhenEmpty, TagInfo, Tags, TargetFilter,
        TargetSelection, TargetsHittable, TargetsInArea, TickBehavior, Ticks,
    },
    actor::{HasHealthBar, IncomingDamageLog},
    area::Fountain,
    camera::Spectatable,
    prelude::{non_damaging::ObjectiveHealthOwner, *},
    GameState,
};

pub struct ArenaPlugin;
impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_arena);
    }
}

pub fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //models: Res<Models>,
) {
    //ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 4.0,
                subdivisions: 5,
            })),
            //material: materials.add(icons.basic_attack.clone().into()),
            transform: Transform {
                translation: Vec3::new(0.0, -0.1, 0.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Static,
        CollisionLayers::GROUND,
        Collider::cuboid(10.0, 0.1, 10.0),
        Name::new("Plane"),
        NavMeshAffector,
    ));

    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(-5.0, 1.0, 0.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Static,
        CollisionLayers::WALL,
        Collider::cuboid(1.0, 2.0, 5.0),
        Name::new("Left Wall"),
        NavMeshAffector,
    ));

    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(5.0, 1.0, 0.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Static,
        CollisionLayers::WALL,
        Collider::cuboid(1.0, 2.0, 5.0),
        Name::new("Right Wall"),
        NavMeshAffector,
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
            Collider::capsule(1.0, 0.7),
            RigidBody::Static,
            CollisionLayers::WALL,
            TEAM_NEUTRAL,
            {
                let mut attributes = Attributes::default();
                attributes
                    .set(Stat::Health, 33.0)
                    .set(Stat::MagicalProtection, 60.0)
                    .set(Stat::PhysicalProtection, 60.0);
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
            Collider::capsule(1.0, 0.5),
            RigidBody::Dynamic,
            LockedAxes::ACTOR,
            CollisionLayers::PLAYER,
            TEAM_2,
            CCMap::default(),
            BuffMap::default(),
            HasHealthBar,
            IncomingDamageLog::default(),
            Spectatable,
            Name::new("Target Dummy"),
        ))
        .insert((Attributes::default(),));

    // Scanning Damage zone RED
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
        Name::new("DamageFountain"),
    )).insert((
        RigidBody::Static,
        Collider::cuboid(4.0, 0.3, 4.0),
        CollisionLayers::ABILITY,
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
    ));

    // Damage zone YELLOW
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
        Spectatable,
        Name::new("DamageFountain2"),
    )).insert((
        RigidBody::Static,
        CollisionLayers::ABILITY,
        Collider::cuboid(4.0, 0.3, 4.0),
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
    ));

    // Fountain GREEN
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
        Spectatable,
        Name::new("Healing Fountain"),
        Fountain,
    )).insert((
        Collider::cuboid(4.0, 0.3, 4.0),
        RigidBody::Static,
        CollisionLayers::ABILITY,
        Sensor,
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
    ));

    // sky
    let _sky = commands
        .spawn((
            SceneBundle {
                //scene: models.skybox.clone(),
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
    /*
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
        */
}
