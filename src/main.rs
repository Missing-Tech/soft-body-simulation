use bevy::{prelude::*, sprite::Wireframe2dPlugin};

#[derive(Component)]
struct Point {
    previous_position: Option<Vec3>,
    mass: f32,
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
struct Line {
    start: Entity,
    end: Entity,
}

fn constrain_to_world(mut query: Query<&mut Transform, With<Point>>) {
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

fn setup_free_line(mut commands: Commands) {
    let stick_length: f32 = 50.0;
    let points_count = 5;
    let mut previous_entity = None;
    for i in 0..=points_count {
        let mass = 50.0;
        let mut cmd = commands.spawn((
            Transform::from_xyz(50.0 * i as f32, 300.0 + (i as f32 * 15.), 0.),
            Point {
                previous_position: None,
                mass,
            },
            Name::new(format!("Point {}", i)),
        ));
        if previous_entity.is_none() {
            cmd.insert((Gravity(0.), Locked));
        }
        let entity = cmd.id();
        if let Some(e) = previous_entity {
            commands.spawn((
                Line {
                    start: e,
                    end: entity,
                },
                DistanceContraint {
                    desired_distance: stick_length - (i as f32 * 5.),
                },
                Name::new(format!("Stick {}", i)),
            ));
        }
        previous_entity = Some(entity);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    let system_set = (verlet_integration, apply_distance_constraint).chain();
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_free_line,
                display_points.after(setup_free_line),
                display_line.after(setup_free_line),
            ),
        )
        .add_systems(Update, update_line)
        .add_systems(FixedUpdate, (system_set, constrain_to_world))
        .run();
}
