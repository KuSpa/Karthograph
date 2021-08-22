use bevy::prelude::*;
use karthograph_core::*;

fn main() {
     App::build()
        .insert_resource(AssetManager::default())
        .add_event::<NewCard>()
        .insert_resource(GameObjectives::default())
        .insert_resource(Season::default())
        .insert_resource(RuinIndicator::default())
        .insert_resource(MousePosition::default())
        .add_plugins(DefaultPlugins)
        .add_asset::<CardPile>()
        .init_asset_loader::<CardPileLoader>()
        .add_startup_system(init_camera.system())
        .add_startup_system(setup_ui.system())
        .add_state(GameState::Loading)
        .add_system_set(SystemSet::on_enter(GameState::Loading).with_system(init_assets.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Loading).with_system(check_readiness.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(init_grid.system()))
        .add_system_set(
            SystemSet::on_enter(GameState::SeasonState)
                .with_system(initialize_cards.system())
                .with_system(setup_objective_ui.system()),
        )
        .add_system_set(
            SystemSet::on_resume(GameState::SeasonState).with_system(initialize_cards.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::SeasonState)
                .with_system(next_card.system())
                .with_system(move_shape.system())
                .with_system(mirror_shape.system())
                .with_system(rotate_shape.system())
                .with_system(place_shape.system())
                .with_system(mouse_position.system())
                .with_system(click_card.system()),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::SeasonScoreState).with_system(score_season.system()),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::SeasonScoreState).with_system(advance_season.system()),
        )
        /*.add_system(
            spawn_shape
                .system()
                .config(|params| params.2 = Some(Timer::new(Duration::from_secs_f32(0.1), false))),
        )*/
        .run();
}
