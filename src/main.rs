use bevy::{prelude::*, sprite::Wireframe2dPlugin};

#[derive(Component)]
struct Point {
    previous_position: Vec3,
}

#[derive(Component)]
struct DistanceContraint {
    desired_distance: u16,
}

#[derive(Component)]
struct Line {
    start: Entity,
    end: Entity,
    length: DistanceContraint,
}

fn constrain_to_world(mut query: Query<&mut Transform, With<Point>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = transform.translation.x.clamp(0.0, 500.0);
        transform.translation.y = transform.translation.y.clamp(0.0, 500.0);
    }
}

fn apply_gravity(mut query: Query<&mut Transform, With<Point>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y -= 1.0;
    }
}

fn verlet_integration(mut query: Query<(&mut Point, &mut Transform)>) {
    for (mut point, mut transform) in query.iter_mut() {
        let temp = transform.translation;
        // Dampen velocity slightly to simulate drag
        let velocity = (transform.translation - point.previous_position) * 0.5;
        transform.translation += velocity;
        point.previous_position = temp;
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let point1 = commands
        .spawn((
            Point {
                previous_position: Vec3::ZERO,
            },
            Transform::from_xyz(100.0, 100.0, 0.0),
        ))
        .id();

    let point2 = commands
        .spawn((
            Point {
                previous_position: Vec3::ZERO,
            },
            Transform::from_xyz(0.0, 150.0, 0.0),
        ))
        .id();

    commands.spawn((
        Line {
            start: point1,
            end: point2,
            length: DistanceContraint {
                desired_distance: 20,
            },
        },
        Transform::default(),
        GlobalTransform::default(),
    ));
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(
            Startup,
            (
                setup,
                display_points.after(setup),
                display_line.after(setup),
            ),
        )
        .add_systems(
            Update,
            (
                apply_gravity.before(constrain_to_world),
                constrain_to_world,
                verlet_integration.before(apply_gravity),
                update_line,
            ),
        )
        .run();
}
