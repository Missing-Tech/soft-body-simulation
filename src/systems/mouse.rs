use bevy::{prelude::*, window::PrimaryWindow};

use crate::components::mouse::{CollideWithMouse, FollowMouse};

pub fn follow_mouse(
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

pub fn collide_with_mouse(
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
