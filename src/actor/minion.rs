//use bevy_mod_wanderlust::{ControllerBundle, ControllerInput, Movement, WanderlustPlugin};
/*
use oxidized_navigation::{
    debug_draw::{DrawNavMesh, DrawPath, OxidizedNavigationDebugDrawPlugin},
    query::find_path,
    NavMesh, NavMeshSettings, OxidizedNavigationPlugin,
};
*/

use crate::game_manager::{TEAM_1};
use crate::prelude::*;
use crate::actor::controller::Controller;

pub struct MinionPlugin;
impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MinionControl>()
            .register_type::<MinionPath>()
            .register_type::<MinionProgress>();
        app.add_event::<SpawnMinionEvent>();

        /*
               app.add_plugins((
                   OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings {
                       cell_width: 0.5,
                       cell_height: 0.5,
                       tile_width: 100,
                       world_half_extents: 250.0,
                       world_bottom_bound: -100.0,
                       max_traversable_slope_radians: (40.0_f32 - 0.1).to_radians(),
                       walkable_height: 20,
                       walkable_radius: 1,
                       step_height: 3,
                       min_region_area: 100,
                       merge_region_area: 500,
                       max_contour_simplification_error: 1.5,
                       max_edge_length: 80,
                       max_tile_generation_tasks: Some(1),
                   }),
                   OxidizedNavigationDebugDrawPlugin,
               ));
        */
        //app.add_plugins(WanderlustPlugin::default());

        app.add_systems(FixedUpdate, (spawn_single_minion, spawn_minion).chain().in_set(InGameSet::Pre));
        app.add_systems(FixedUpdate, (minion_follow_path, minion_update_progress).chain().in_set(InGameSet::Update));
        //app.add_systems(Update, (toggle_nav_mesh_system, run_blocking_pathfinding));
    }
}

//
//  Toggle drawing Nav-mesh.
//  Press M to toggle drawing the navmesh.
//
/*
fn toggle_nav_mesh_system(keys: Res<Input<KeyCode>>, mut show_navmesh: ResMut<DrawNavMesh>) {
    if keys.just_pressed(KeyCode::M) {
        show_navmesh.0 = !show_navmesh.0;
    }
}
*/

fn spawn_single_minion(keys: Res<Input<KeyCode>>, mut events: EventWriter<SpawnMinionEvent>) {
    if keys.just_pressed(KeyCode::Semicolon) {
        events.send(SpawnMinionEvent {
            location: Vec3::ZERO,
        });
    }
}

//
//  Blocking Pathfinding.
//  Press B to run.
//
//  Running pathfinding in a system.
//
/*
fn run_blocking_pathfinding(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
) {
    if !keys.just_pressed(KeyCode::B) {
        return;
    }

    // Get the underlying nav_mesh.
    if let Ok(nav_mesh) = nav_mesh.get().read() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(10.0, 0.0, 0.0);

        match find_path(
            &nav_mesh,
            &nav_mesh_settings,
            start,
            end,
            None,
            Some(&[1.0, 0.5]),
        ) {
            Ok(path) => {
                info!("Path found (BLOCKING): {:?}", path);

                commands.spawn(DrawPath {
                    timer: Some(Timer::from_seconds(4.0, TimerMode::Once)),
                    pulled_path: path,
                    color: Color::RED,
                });
            }
            Err(error) => error!("Error with pathfinding: {:?}", error),
        }
    }
}
*/

#[derive(Event, Copy, Clone)]
pub struct SpawnMinionEvent {
    pub location: Vec3,
}

pub fn spawn_minion(mut commands: Commands, mut spawns: EventReader<SpawnMinionEvent>) {
    for spawn in spawns.read() {
        let minion_collider = commands
            .spawn(SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.51, 0.0),
                    ..default()
                },
                ..default()
            })
            .insert(Collider::capsule(0.5, 0.3))
            .id();

        let steps = 8;
        let radius = 2.0;
        let mut points = Vec::new();

        for i in 0..steps {
            let theta = i as f32 / steps as f32 * std::f32::consts::TAU;
            let x = theta.cos();
            let y = theta.sin();
            points.push(Vec3::new(x, 0.0, y) * radius);
        }

        #[allow(unused_variables)]
        let circle_patrol = MinionPath {
            points: points,
            patrol: true,
        };

        let defined_path = MinionPath {
            points: vec![
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 7.0),
                Vec3::new(7.0, 0.0, 7.0),
                Vec3::new(7.0, 0.0, -7.0),
                Vec3::new(0.0, 0.0, -12.0),
            ],
            patrol: false,
        };

        commands
            .spawn(SpatialBundle::from_transform(Transform::from_translation(
                spawn.location,
            )))
            .insert(Name::new("Minion"))
            //.insert(ControllerBundle { ..default() })
            .insert(Controller {
                max_speed: 1.0,
                ..default()
            })
            .insert((
                defined_path,
                //circle_patrol,
                MinionProgress::default(),
                MinionControl {
                    movement: Vec3::ZERO,
                },
            ))
            .insert(
                // physics
                (RigidBody::Dynamic, LockedAxes::ACTOR, CollisionLayers::PLAYER),
            )
            .add_child(minion_collider)
            .insert({
                let mut attrs = Attributes::default();
                attrs
                    .set_base(Stat::Health, 50.0)
                    .set_base(Stat::Speed, 3.0);
                attrs
            })
            .insert((
                ActorType::Minion,
                ActorState::Alive,
                TEAM_1,
                //CCMap::default(),
                //BuffMap::default(),
            ));
    }
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MinionPath {
    points: Vec<Vec3>,
    patrol: bool,
}

impl MinionPath {
    pub fn closest_point(&self, from: Vec3) -> Option<usize> {
        self.points
            .iter()
            .enumerate()
            .map(|(index, x)| (index, x.distance(from)))
            .min_by(|(_, a), (_, b)| (*b).partial_cmp(a).unwrap_or(std::cmp::Ordering::Less))
            .map(|(index, _dist)| index)
    }

    pub fn points(&self) -> &[Vec3] {
        self.points.as_slice()
    }

    pub fn get(&self, progress: usize) -> Option<Vec3> {
        self.points.get(progress).map(|v| Vec3::new(v.x, 0.0, v.z))
    }
}

/// Simple index of the current path point we are heading towards.
#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MinionProgress {
    pub previous: usize,
    pub next: usize,
}

#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MinionControl {
    pub movement: Vec3,
}

pub fn minion_follow_path(
    mut minions: Query<(
        &GlobalTransform,
        &mut Controller,
        &MinionPath,
        &MinionProgress,
        &Attributes,
    )>,

    mut gizmos: Gizmos,
) {
    for (position, mut control, path, progress, attributes) in &mut minions {
        for points in path.points().windows(2) {
            let start = points[0];
            let end = points[1];
            gizmos.line(start, end, Color::RED);
        }

        let mut position = position.translation();
        position.y = 0.0;

        control.direction = Vec3::ZERO;

        let Some(next_point) = path.points.get(progress.next) else {
            continue;
        };
        let difference = *next_point - position;
        let direction = difference.normalize_or_zero();
        //info!("direction: {:.1?}", direction);
        control.direction = direction;
        control.max_speed = attributes.get(Stat::Speed);
    }
}

pub fn minion_update_progress(
    mut minions: Query<(&GlobalTransform, &MinionPath, &mut MinionProgress)>,
) {
    for (position, path, mut progress) in &mut minions {
        let mut position = position.translation();
        position.y = 0.0;

        let (prev_point, next_point) = if progress.next == progress.previous {
            let Some(next_point) = path.points.get(progress.next) else {
                continue;
            };

            (position, *next_point)
        } else {
            let Some(prev_point) = path.points.get(progress.previous) else {
                continue;
            };
            let Some(next_point) = path.points.get(progress.next) else {
                continue;
            };

            (*prev_point, *next_point)
        };

        let relative_position = position - prev_point;
        let position_difference = next_point - position;
        let path_difference = next_point - prev_point;
        let path_direction = path_difference.normalize_or_zero();

        let projected = relative_position.dot(path_direction);

        if projected >= path_difference.length() - 0.2 {
            if position_difference.length() < 2.0 {
                progress.previous = progress.next;
                progress.next += 1;

                if path.patrol && path.points.len() > 0 && progress.next >= path.points.len() - 1 {
                    progress.next = 0;
                }
            }
        }
    }
}
