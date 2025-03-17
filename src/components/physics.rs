use bevy::prelude::*;

#[derive(Component)]
pub struct DistanceContraint {
    pub desired_distance: f32,
}

#[derive(Component)]
pub struct AreaConstraint {
    pub desired_area: f32,
}

#[derive(Component)]
pub struct Gravity(pub f32);

#[derive(Component)]
pub struct Friction(pub f32);

#[derive(Component)]
pub struct Locked;
