use bevy::prelude::*;
use bevy_prototype_lyon::{
    draw::Fill, entity::ShapeBundle, path::PathBuilder, prelude::GeometryBuilder,
};

use crate::components::shapes::{Blob, BlobShape, Line, Point};

pub fn draw_blob(
    mut commands: Commands,
    mut blob_query: Query<&Blob>,
    old_shapes: Query<Entity, With<BlobShape>>,
    point_query: Query<(&Transform, &Point), (With<Point>, Without<Line>, Without<Blob>)>,
) {
    for entity in old_shapes.iter() {
        commands.entity(entity).despawn();
    }

    for blob in blob_query.iter_mut() {
        let mut positions = Vec::with_capacity(blob.points.len());
        for &entity in &blob.points {
            if let Ok((transform, _)) = point_query.get(entity) {
                positions.push(transform.translation.truncate());
            }
        }

        if positions.len() > 1 {
            let spline = CubicCardinalSpline::new(0.5, positions)
                .to_curve_cyclic()
                .unwrap();
            let mut domain = spline.domain().spaced_points(100).unwrap();
            let mut path_builder = PathBuilder::new();

            let start = spline.sample(domain.next().unwrap()).unwrap();
            path_builder.move_to(start);

            for t in domain.into_iter() {
                let p = spline.sample(t);
                if p.is_some() {
                    path_builder.line_to(p.unwrap());
                }
            }

            path_builder.close();

            let path = path_builder.build();

            commands.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&path),
                    transform: Transform::default(),
                    visibility: default(),
                    ..default()
                },
                Fill::color(Color::srgb(0.42, 0.44, 0.53)),
                BlobShape,
            ));
        }
    }
}
