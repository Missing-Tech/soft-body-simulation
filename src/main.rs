use std::f32::consts::{PI, TAU};

use bevy::{prelude::*, sprite::Wireframe2dPlugin, window::PrimaryWindow};

#[derive(Component)]
struct Point {
    previous_position: Option<Vec3>,
    mass: f32,
}

#[derive(Component)]
struct Blob {
    lines: Vec<Entity>,
}

#[derive(Component)]
struct DistanceContraint {
    desired_distance: f32,
}

#[derive(Component)]
struct Gravity(f32);

#[derive(Component)]
struct Friction(f32);

#[derive(Component)]
struct Locked;

#[derive(Component)]
struct FollowMouse;

#[derive(Component)]
struct Line {
    start: Entity,
    end: Entity,
}

fn constrain_to_world(
    mut query: Query<&mut Transform, Or<(With<Point>, With<FollowMouse>)>>,
    mut gizmos: Gizmos,
) {
    gizmos.rect_2d(
        Isometry2d::from_translation(Vec2::new(250.0, 250.0)),
        Vec2::new(500.0, 500.0),
        Color::WHITE,
    );
    for mut transform in query.iter_mut() {
        transform.translation.x = transform.translation.x.clamp(0.0, 500.0);
        transform.translation.y = transform.translation.y.clamp(0.0, 500.0);
    }
}

fn apply_distance_constraint(
    mut line_query: Query<(&Line, &DistanceContraint)>,
    mut point_query: Query<(&mut Transform, Option<&Locked>), (With<Point>, Without<Line>)>,
) {
    for (line, distance_constraint) in line_query.iter_mut() {
        if let Ok([(mut start_tf, start_locked), (mut end_tf, end_locked)]) =
            point_query.get_many_mut([line.start, line.end])
        {
            let (start_locked, end_locked) = (start_locked.is_some(), end_locked.is_some());
            if start_locked && end_locked {
                continue;
            }

            for _ in 0..=5 {
                let start_pos = start_tf.translation;
                let end_pos = end_tf.translation;

                let midpoint = (start_pos + end_pos) / 2.0;
                let direction = (end_pos - start_pos).normalize();
                let new_vector = direction * distance_constraint.desired_distance / 2.0;

                if !start_locked {
                    start_tf.translation = if end_locked {
                        end_tf.translation + new_vector * 2.0
                    } else {
                        midpoint + new_vector
                    };
                }

                if !end_locked {
                    end_tf.translation = if start_locked {
                        start_tf.translation - new_vector * 2.0
                    } else {
                        midpoint - new_vector
                    }
                }
            }
        }
    }
}

fn verlet_integration(
    mut query: Query<(
        &mut Point,
        &mut Transform,
        Option<&Gravity>,
        Option<&Friction>,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut point, mut transform, gravity, friction) in query.iter_mut() {
        let position = transform.translation;
        let velocity = point
            .previous_position
            .map_or(Vec3::ZERO, |pos| position - pos);
        let acceleration = Vec3::new(0.0, gravity.unwrap_or(&Gravity(-9.81)).0, 0.0) * point.mass;
        let friction = friction.unwrap_or(&Friction(0.99)).0;

        transform.translation += velocity * friction + acceleration * dt * dt;
        point.previous_position = Some(position);
    }
}

fn update_line(
    mut line_query: Query<(&Line, &mut Transform)>,
    point_query: Query<&Transform, (With<Point>, Without<Line>)>,
) {
    for (line, mut transform) in line_query.iter_mut() {
        if let (Ok(start_tf), Ok(end_tf)) = (point_query.get(line.start), point_query.get(line.end))
        {
            let start_pos = start_tf.translation;
            let end_pos = end_tf.translation;

            let length = start_pos.distance(end_pos);
            let midpoint = (start_pos + end_pos) / 2.0;
            let direction = end_pos - start_pos;
            let angle = direction.y.atan2(direction.x);

            transform.translation = Vec3::new(midpoint.x, midpoint.y, -1.0);
            transform.rotation = Quat::from_rotation_z(angle);
            transform.scale = Vec3::new(length, 1.0, 1.0);
        }
    }
}

fn display_line(
    query: Query<Entity, With<Line>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let quad = Rectangle::new(1.0, 3.0);
    let mesh_handle = meshes.add(quad);
    let color_handle = materials.add(ColorMaterial::from(Color::WHITE));

    for entity in query.iter() {
        commands.entity(entity).insert_if_new((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(color_handle.clone()),
        ));
    }
}

fn display_points(
    query: Query<Entity, With<Point>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle = meshes.add(Circle::new(8.0));
    let colour = materials.add(ColorMaterial::from(Color::WHITE));

    for entity in query.iter() {
        commands
            .entity(entity)
            .insert((Mesh2d(circle.clone()), MeshMaterial2d(colour.clone())));
    }
}

fn follow_mouse(
    windows_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform, With<FollowMouse>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Only update positions if the left mouse button is currently pressed.
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = windows_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            Ok(world_position) => {
                for mut transform in query.iter_mut() {
                    transform.translation.x = world_position.x;
                    transform.translation.y = world_position.y;
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}

fn setup_free_line(mut commands: Commands) {
    let stick_length: f32 = 20.0;
    let points_count = 30;
    let mut previous_entity = None;
    for i in 0..=points_count {
        let mass = 50.0;
        let mut cmd = commands.spawn((
            Transform::from_xyz(20.0 * i as f32, 300.0, 0.),
            Point {
                previous_position: None,
                mass,
            },
            Name::new(format!("Point {}", i)),
        ));
        if previous_entity.is_none() {
            cmd.insert((FollowMouse, Locked));
        }
        let entity = cmd.id();
        if let Some(e) = previous_entity {
            commands.spawn((
                Line {
                    start: e,
                    end: entity,
                },
                DistanceContraint {
                    desired_distance: stick_length,
                },
                Name::new(format!("Stick {}", i)),
            ));
        }
        previous_entity = Some(entity);
    }
}

fn setup_blob(mut commands: Commands) {
    let radius = 50.0;
    let circumference = radius * PI * 2.0;
    let num_points = 8;
    let chord_length = circumference / num_points as f32;

    let mut previous_entity = None;
    let mut first_entity = None;

    let mut lines = Vec::new();

    for i in 0..num_points {
        let mass = 50.0;
        let angle = TAU * (i as f32 / num_points as f32) - PI / 2.0;
        let offset = Vec2::new(angle.cos() * radius, angle.sin() * radius);

        let mut cmd = commands.spawn((
            Transform::from_xyz(offset.x + 250.0, offset.y + 250.0, 0.0),
            Point {
                previous_position: None,
                mass,
            },
        ));

        if previous_entity.is_none() {
            cmd.insert(FollowMouse);
            first_entity = Some(cmd.id());
        }

        let entity = cmd.id();

        if let Some(e) = previous_entity {
            let line_entity = commands
                .spawn((
                    Line {
                        start: e,
                        end: entity,
                    },
                    DistanceContraint {
                        desired_distance: chord_length,
                    },
                ))
                .id();
            lines.push(line_entity);
        }

        previous_entity = Some(entity);
    }

    // Finally, connect the last point to the first, to close the loop.
    if let (Some(first), Some(last)) = (first_entity, previous_entity) {
        if first != last {
            let line_entity = commands
                .spawn((
                    Line {
                        start: last,
                        end: first,
                    },
                    DistanceContraint {
                        desired_distance: chord_length,
                    },
                ))
                .id();
            lines.push(line_entity);
        }
    }

    // Create a single “Blob” entity referencing all line entities.
    commands.spawn((Blob { lines }, Name::new("Blob")));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(250., 250., 0.)));
}

fn main() {
    let system_set = (
        verlet_integration,
        follow_mouse,
        apply_distance_constraint,
        constrain_to_world,
    )
        .chain();
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_blob,
                display_points.after(setup_blob),
                display_line.after(setup_blob),
            ),
        )
        .add_systems(Update, update_line)
        .add_systems(FixedUpdate, system_set)
        .run();
}
