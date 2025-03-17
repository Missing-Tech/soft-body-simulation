use bevy::prelude::*;

#[derive(Component)]
pub struct Point {
    pub previous_position: Option<Vec3>,
    pub mass: f32,
    pub displacement: Vec3,
    pub displacement_weight: u32,
}

#[derive(Component)]
pub struct Blob {
    pub points: Vec<Entity>,
    pub circumference: f32,
}

#[derive(Component)]
pub struct BlobShape;

#[derive(Component)]
pub struct Line {
    pub start: Entity,
    pub end: Entity,
}
