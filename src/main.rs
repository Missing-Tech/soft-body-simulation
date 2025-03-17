use std::f32::consts::{PI, TAU};

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct Point {
    previous_position: Option<Vec3>,
    mass: f32,
    displacement: Vec3,
    displacement_weight: u32,
}

#[derive(Component)]
struct Blob {
    points: Vec<Entity>,
    circumference: f32,
}

#[derive(Component)]
struct DistanceContraint {
    desired_distance: f32,
}

#[derive(Component)]
struct AreaConstraint {
    desired_area: f32,
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
struct CollideWithMouse;

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
    mut point_query: Query<(&mut Point, &Transform, Option<&Locked>), Without<Line>>,
) {
    for (line, distance_constraint) in line_query.iter_mut() {
        if let Ok(
            [(mut start_point, start_tf, start_locked), (mut end_point, end_tf, end_locked)],
        ) = point_query.get_many_mut([line.start, line.end])
        {
            let (start_locked, end_locked) = (start_locked.is_some(), end_locked.is_some());
            if start_locked && end_locked {
                continue;
            }
            for _ in 0..5 {
                let start_pos = start_tf.translation;
                let end_pos = end_tf.translation;

                let midpoint = (start_pos + end_pos) / 2.0;
                let direction = (end_pos - start_pos).normalize();
                let new_vector = direction * distance_constraint.desired_distance / 2.0;

                if !start_locked {
                    let new_start = if end_locked {
                        end_tf.translation + new_vector * 2.0
                    } else {
                        midpoint + new_vector
                    };
                    start_point.displacement += new_start - start_pos;
                    start_point.displacement_weight += 1;
                }

                if !end_locked {
                    let new_end = if start_locked {
                        start_tf.translation - new_vector * 2.0
                    } else {
                        midpoint - new_vector
                    };
                    end_point.displacement += new_end - end_pos;
                    end_point.displacement_weight += 1;
                }
            }
        }
    }
}

fn get_trapezoidal_area(points: &[Vec3]) -> f32 {
    let mut area: f32 = 0.;

    for i in 0..points.len() {
        let current = points[i];
        let next = if i == points.len() - 1 {
            points[0]
        } else {
            points[i + 1]
        };

        area += (current.x - next.x) * (current.y + next.y) / 2.;
    }

    return area.abs();
}

fn apply_area_constraint(
    mut blob_query: Query<(&Blob, &AreaConstraint)>,
    mut point_query: Query<
        (&mut Transform, &mut Point),
        (With<Point>, Without<Line>, Without<Blob>),
    >,
) {
    for (blob, area_constraint) in blob_query.iter_mut() {
        for _ in 0..5 {
            let mut positions = Vec::with_capacity(blob.points.len());
            for &entity in &blob.points {
                if let Ok((transform, _)) = point_query.get(entity) {
                    positions.push(transform.translation);
                } else {
                    positions.push(Vec3::ZERO);
                }
            }

            let current_area = get_trapezoidal_area(&positions);

            let error = area_constraint.desired_area - current_area;
            let offset = error / blob.circumference;

            let len = positions.len();
            for i in 0..len {
                let prev_idx = if i == 0 { len - 1 } else { i - 1 };
                let next_idx = (i + 1) % len;

                let prev_pos = positions[prev_idx];
                let next_pos = positions[next_idx];

                let secant = next_pos - prev_pos;
                let normal = Vec3::new(secant.y, -secant.x, 0.0).normalize_or_zero() * offset;

                if let Ok((_, mut point)) = point_query.get_mut(blob.points[i]) {
                    point.displacement += normal;
                    point.displacement_weight += 1;
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

fn apply_displacement(mut query: Query<(&mut Transform, &mut Point)>) {
    for (mut transform, mut point) in query.iter_mut() {
        if point.displacement_weight > 0 {
            let displacement = point.displacement / point.displacement_weight as f32;

            transform.translation += displacement;

            point.displacement = Vec3::ZERO;
            point.displacement_weight = 0;
        }
    }
}

fn draw_blob(
    mut blob_query: Query<&Blob>,
    point_query: Query<(&Transform, &Point), (With<Point>, Without<Line>, Without<Blob>)>,
    mut gizmos: Gizmos,
) {
    for blob in blob_query.iter_mut() {
        let mut positions = Vec::with_capacity(blob.points.len());
        for &entity in &blob.points {
            if let Ok((transform, _)) = point_query.get(entity) {
                positions.push(transform.translation.truncate());
            }
        }

        if positions.len() > 1 {
            let bezier = CubicCardinalSpline::new(0.5, positions)
                .to_curve_cyclic()
                .unwrap();

            let segments = bezier.domain().spaced_points(100).unwrap();

            gizmos.curve_2d(bezier, segments, Color::WHITE);
        }
    }
}

fn collide_with_mouse(
    windows_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform, With<CollideWithMouse>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut gizmos: Gizmos,
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
                gizmos.circle_2d(
                    Isometry2d::from_xy(world_position.x, world_position.y),
                    50.,
                    Color::WHITE,
                );
                for mut transform in query.iter_mut() {
                    let vec3_world_pos = Vec3::new(world_position.x, world_position.y, 0.);
                    let distance = transform.translation.distance(vec3_world_pos);
                    if distance < 50. {
                        let difference = (transform.translation - vec3_world_pos).normalize() * 50.;
                        transform.translation = vec3_world_pos + difference;
                    }
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
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

fn setup_blob(mut commands: Commands) {
    let radius = 50.0;
    let circumference = radius * PI * 2.0;
    let area = radius * radius * PI * 1.2;
    let num_points = 16;
    let chord_length = circumference / num_points as f32;

    let mut previous_entity = None;
    let mut first_entity = None;

    let mut points = Vec::new();

    for i in 0..num_points {
        let mass = 100.0;
        let angle = TAU * (i as f32 / num_points as f32) - PI / 2.0;
        let offset = Vec2::new(angle.cos() * radius, angle.sin() * radius);

        let cmd = commands.spawn((
            Transform::from_xyz(offset.x + 250.0, offset.y + 250.0, 0.0),
            Point {
                previous_position: None,
                mass,
                displacement: Vec3::ZERO,
                displacement_weight: 0,
            },
            Friction(0.9),
            CollideWithMouse,
        ));

        if previous_entity.is_none() {
            //cmd.insert(FollowMouse);
            first_entity = Some(cmd.id());
        }

        let entity = cmd.id();
        points.push(entity);

        if let Some(e) = previous_entity {
            commands.spawn((
                Line {
                    start: e,
                    end: entity,
                },
                DistanceContraint {
                    desired_distance: chord_length,
                },
            ));
        }

        previous_entity = Some(entity);
    }

    if let (Some(first), Some(last)) = (first_entity, previous_entity) {
        if first != last {
            commands.spawn((
                Line {
                    start: last,
                    end: first,
                },
                DistanceContraint {
                    desired_distance: chord_length,
                },
            ));
        }
    }

    commands.spawn((
        Blob {
            points,
            circumference,
        },
        AreaConstraint { desired_area: area },
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(250., 250., 0.)));
}

fn main() {
    let system_set = (
        verlet_integration,
        follow_mouse,
        collide_with_mouse,
        apply_distance_constraint,
        apply_area_constraint,
        apply_displacement,
        constrain_to_world,
    )
        .chain();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_blob))
        .add_systems(Update, draw_blob)
        .add_systems(FixedUpdate, system_set)
        .run();
}
