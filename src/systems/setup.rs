use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::components::{
    mouse::CollideWithMouse,
    physics::{AreaConstraint, DistanceContraint, Friction},
    shapes::{Blob, Line, Point},
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(250., 250., 0.)));
}

pub fn setup_blob(mut commands: Commands) {
    let radius = 80.0;
    let circumference = radius * PI * 2.0;
    let area = radius * radius * PI * 1.2;
    let num_points = 24;
    let chord_length = circumference / num_points as f32;

    let mut previous_entity = None;
    let mut first_entity = None;

    let mut points = Vec::new();

    for i in 0..num_points {
        let mass = 150.0;
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
            Friction(0.95),
            CollideWithMouse,
        ));

        if previous_entity.is_none() {
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
