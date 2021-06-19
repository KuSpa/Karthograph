use std::time::Duration;
use std::usize;

use bevy::core::Timer;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;

mod asset_management;
mod grid;
mod shape;
mod util;
use grid::*;
use shape::*;

use asset_management::{init_assets};

pub const SPRITE_SIZE: f32 = 75.;
pub const GRID_SIZE: usize = 11;
//x=y offset
pub const GRID_OFFSET: f32 = SPRITE_SIZE;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system_to_stage(StartupStage::PreStartup, init_assets.system())
        .add_startup_system(init_camera.system())
        .add_startup_system(init_grid.system())
        .add_system(move_shape.system())
        .add_system(place_shape.system())
        .add_system(rotate_shape.system())
        .add_system(
            spawn_shape
                .system()
                .config(|params| params.2 = Some(Timer::new(Duration::from_secs_f32(0.1), false))),
        )
        .run();
}

fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}
