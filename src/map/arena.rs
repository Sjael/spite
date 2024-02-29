use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use crate::{
    ability::{
        ticks::{PausesWhenEmpty, TickBehavior},
        Ability, TagInfo, Tags, TargetFilter, TargetSelection, TargetsHittable, TargetsInArea,
    },
    actor::{
        cast::{Caster, Tower},
        HasHealthBar, IncomingDamageLog,
    },
    buff::{BuffInfo, BuffMap, BuffTargets, BuffType},
    camera::Spectatable,
    crowd_control::{CCInfo, CCKind, CCMap},
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
    icons: Res<Icons>,
    //models: Res<Models>,
) {
    //ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(4.0, 4.0)),
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
            meshes.add(Capsule3d::new(0.7, 2.0)),
            materials.add(StandardMaterial::from(Color::RED)),
            Collider::capsule(1.0, 0.7),
            RigidBody::Static,
            CollisionLayers::WALL,
            TEAM_NEUTRAL,
            ActorState::Alive,
            {
                let mut attributes = Attributes::default();
                attributes
                    .set(Stat::Health, 100.0)
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
        .spawn((SpatialBundle::default(), Name::new("Range Collider")))
        .insert((
            Collider::cylinder(1.0, 7.),
            Sensor,
            TickBehavior::new_static(2.0),
            PausesWhenEmpty,
            Caster(tower),
            TargetFilter {
                number_of_targets: 1,
                target_selection: TargetSelection::Closest,
                ..default()
            },
            TargetsHittable::default(),
            TargetsInArea::default(),
            Tags(vec![TagInfo::Homing(Ability::Fireball)]),
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
            meshes.add(Capsule3d::new(0.4, 1.0)),
            materials.add(StandardMaterial::from(Color::INDIGO)),
            Collider::capsule(1.0, 0.5),
            RigidBody::Dynamic,
            LockedAxes::ACTOR,
            CollisionLayers::PLAYER,
            TEAM_2,
            ActorState::Alive,
            CCMap::default(),
            BuffMap::default(),
            HasHealthBar,
            IncomingDamageLog::default(),
            Spectatable,
            Name::new("Target Dummy"),
        ))
        .insert((ActorType::Minion,))
        .insert({
            let mut attrs = Attributes::default();
            attrs
                .set(Stat::Health, 200.0)
                .set_base(Stat::Speed, 6.0)
                .set(Stat::CharacterResource, 0.0);
            attrs
        });

    // Scanning Damage zone RED
    commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(-10.0, 0.0, 10.0),
                ..default()
            }),
            meshes.add(Plane3d::default().mesh().size(4.0, 4.0)),
            materials.add(StandardMaterial::from(Color::MAROON)),
            Name::new("DamageFountain"),
        ))
        .insert((
            RigidBody::Static,
            Collider::cuboid(4.0, 0.3, 4.0),
            CollisionLayers::ABILITY,
            Sensor,
            TickBehavior::new_static(5.0),
            TargetsInArea::default(),
            TargetsHittable::default(),
            TEAM_NEUTRAL,
            Tags(vec![
                TagInfo::Damage(0.0),
                TagInfo::CC(CCInfo {
                    cckind: CCKind::Stun,
                    duration: 3.0,
                }),
            ]),
        ));

    // Damage zone YELLOW
    commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(10.0, 0.0, 10.0),
                ..default()
            }),
            meshes.add(Plane3d::default().mesh().size(4.0, 4.0)),
            materials.add(StandardMaterial::from(Color::GOLD)),
            Spectatable,
            Name::new("DamageFountain2"),
        ))
        .insert((
            RigidBody::Static,
            CollisionLayers::ABILITY,
            Collider::cuboid(4.0, 0.3, 4.0),
            Sensor,
            Tags(vec![
                TagInfo::Damage(27.0),
                TagInfo::Buff(BuffInfo {
                    stat: AttributeTag::Modifier {
                        modifier: Modifier::Mul,
                        target: Box::new(Stat::Speed.into()),
                    },
                    amount: 20.0,
                    max_stacks: 3,
                    duration: 10.0,
                    image: Ability::Fireball.get_image(&icons),
                    ..default()
                }),
                TagInfo::Buff(BuffInfo {
                    stat: AttributeTag::Modifier {
                        modifier: Modifier::Add,
                        target: Box::new(Stat::CharacterResourceMax.into()),
                    },
                    amount: 1.0,
                    max_stacks: 3,
                    duration: 10.0,
                    image: Ability::Frostbolt.get_image(&icons),
                    ..default()
                }),
                TagInfo::CC(CCInfo {
                    cckind: CCKind::Silence,
                    duration: 7.0,
                }),
            ]),
            TickBehavior::new_individual(1.0),
            TEAM_NEUTRAL,
            TargetsInArea::default(),
            TargetsHittable::default(),
        ));

    // Fountain GREEN
    commands
        .spawn((
            SpatialBundle::from_transform(Transform {
                translation: Vec3::new(10.0, 0.0, -10.0),
                ..default()
            }),
            meshes.add(Plane3d::default().mesh().size(4.0, 4.0)),
            materials.add(StandardMaterial::from(Color::GREEN)),
            Spectatable,
            Name::new("Healing Fountain"),
            Fountain,
        ))
        .insert((
            Collider::cuboid(4.0, 0.3, 4.0),
            RigidBody::Static,
            CollisionLayers::ABILITY,
            Sensor,
            TEAM_1,
            TargetsInArea::default(),
            TargetsHittable::default(),
            TickBehavior::new_individual(1.0),
            Tags(vec![
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
                    image: Ability::Dash.get_image(&icons),
                    ..default()
                }),
            ]),
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
