use bevy::{prelude::*, sprite::Wireframe2dPlugin};

#[derive(Component)]
struct Point {
    previous_position: Vec3,
}

fn constrain_to_world(mut query: Query<(&Point, &mut Transform)>) {
    for (_, mut transform) in &mut query {
        transform.translation.x = transform.translation.x.clamp(0.0, 50.0);
        transform.translation.y = transform.translation.y.clamp(0.0, 50.0);
    }
}

fn verlet_integration(mut query: Query<(&mut Point, &mut Transform)>) {
    for (mut point, mut transform) in &mut query {
        let temp_position = transform.translation.clone();
        // slightly dampen the velocity to simulate drag
        let velocity = (transform.translation - point.previous_position) * 0.99;
        transform.translation += velocity;
        point.previous_position = temp_position;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle = meshes.add(Circle::new(10.0));
    let colour = materials.add(Color::srgb(1.0, 0.0, 0.0));

    commands.spawn(Camera2d);
    commands.spawn((
        Point {
            previous_position: Vec3::NEG_X,
        },
        Transform {
            ..Default::default()
        },
        Mesh2d(circle),
        MeshMaterial2d(colour),
    ));
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (verlet_integration, constrain_to_world))
        .run();
}
