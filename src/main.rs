use asset_management::AssetManager;
use asset_management::check_readiness;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;
use card::RuinIndicator;
use seasons::PassedTime;
use std::usize;

mod asset_management;
mod card;
mod card_pile;
mod grid;
mod shape;
mod util;
use card_pile::*;
use grid::*;
use shape::*;
mod mouse;
mod seasons;
use card::click_card;
use mouse::*;

use asset_management::init_assets;

pub const SPRITE_SIZE: f32 = 75.;
pub const GRID_SIZE: usize = 11;
//x=y offset
pub const GRID_OFFSET: f32 = SPRITE_SIZE;

fn main() {
    App::build()
        .insert_resource(AssetManager::default())
        .insert_resource(RuinIndicator::default())
        .insert_resource(PassedTime::default())
        .insert_resource(MousePosition::default())
        .add_plugins(DefaultPlugins)
        .add_asset::<CardPile>()
        .init_asset_loader::<CardPileLoader>()
        .add_state(GameState::Loading)
        .add_system_set(SystemSet::on_enter(GameState::Loading).with_system(init_assets.system()))
        .add_system_set(SystemSet::on_update(GameState::Loading).with_system(check_readiness.system()))
        .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(init_grid.system()))
        .add_system_set(
            SystemSet::on_enter(GameState::SeasonState).with_system(initialize_cards.system()),
        )        
        .add_system_set(
            SystemSet::on_update(GameState::SeasonState)
                .with_system(next_card.system())
                .with_system(move_shape.system())
                .with_system(rotate_shape.system())
                .with_system(place_shape.system())
                .with_system(mouse_position.system())
                .with_system(click_card.system()).with_system(initialize_cards.system())
        )
        .add_startup_system(init_camera.system())
        /*.add_system(
            spawn_shape
                .system()
                .config(|params| params.2 = Some(Timer::new(Duration::from_secs_f32(0.1), false))),
        )*/
        .run();
}

fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Loading,
    SeasonState,
    SeasonScoreState,
}
