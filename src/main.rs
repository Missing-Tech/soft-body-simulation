use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use systems::{
    display::draw_blob,
    mouse::{collide_with_mouse, follow_mouse},
    physics::{
        apply_area_constraint, apply_displacement, apply_distance_constraint, constrain_to_world,
        verlet_integration,
    },
    setup::{setup_blob, setup_camera},
};

mod components;
mod systems;
mod util;

fn main() {
    let system_set = (
        verlet_integration,
        follow_mouse,
        collide_with_mouse,
        apply_distance_constraint,
        apply_area_constraint,
        apply_displacement,
        constrain_to_world,
    )
        .chain();
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin))
        .add_systems(Startup, (setup_camera, setup_blob))
        .add_systems(Update, draw_blob)
        .add_systems(FixedUpdate, system_set)
        .run();
}
