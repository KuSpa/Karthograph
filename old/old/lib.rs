

use bevy::{prelude::*, render::camera::WindowOrigin};
pub const SPRITE_SIZE: f32 = 75.;
pub const GRID_SIZE: usize = 11;
//x=y offset
pub const GRID_OFFSET: f32 = SPRITE_SIZE;

pub fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Loading,
    SeasonState,
    SeasonScoreState,
    End,
}