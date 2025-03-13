use avian2d::prelude::*;
use bevy::{math::vec2, prelude::*};

#[derive(Component)]
struct Ball;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .insert_resource(Gravity(Vec2::NEG_Y * 100.0))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (setup, spawn_ball))
        .run();
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(50.0));
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.0, 0.0)));
    commands.spawn((
        Ball,
        Mesh2d(shape),
        MeshMaterial2d(material),
        RigidBody::Dynamic,
        Collider::circle(50.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
