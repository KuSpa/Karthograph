pub use asset_management::{check_readiness, init_assets, AssetManager};
pub use card::{click_card, RuinIndicator};
pub use card_pile::*;
pub use grid::*;
pub use mouse::*;
pub use objective::GameObjectives;
pub use seasons::{advance_season, score_season, Season};
pub use shape::*;
pub use ui::{setup_objective_ui, setup_ui};
use bevy::{prelude::*, render::camera::WindowOrigin};

mod asset_management;
mod card;
mod card_pile;
mod grid;
mod mouse;
mod objective;
mod seasons;
mod shape;
mod ui;
mod util;

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
