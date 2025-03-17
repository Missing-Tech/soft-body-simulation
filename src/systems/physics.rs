use bevy::prelude::*;

use crate::{
    components::{
        mouse::FollowMouse,
        physics::{AreaConstraint, DistanceContraint, Friction, Gravity, Locked},
        shapes::{Blob, Line, Point},
    },
    util::shapes::get_trapezoidal_area,
};

pub fn constrain_to_world(
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

pub fn apply_distance_constraint(
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

pub fn apply_area_constraint(
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

pub fn verlet_integration(
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

pub fn apply_displacement(mut query: Query<(&mut Transform, &mut Point)>) {
    for (mut transform, mut point) in query.iter_mut() {
        if point.displacement_weight > 0 {
            let displacement = point.displacement / point.displacement_weight as f32;

            transform.translation += displacement;

            point.displacement = Vec3::ZERO;
            point.displacement_weight = 0;
        }
    }
}
