use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};
use cards::CardPlugin;
use state::Players;
use state::StatePlugin;

mod cards;
mod grid;
mod state;

fn main() {
    let mut app = App::new();
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
        1000 / 30,
    )))
    .add_plugins(MinimalPlugins)
    .add_plugin(StatePlugin)
    .insert_resource(Players::default())
    .add_plugin(LogPlugin)
    .add_plugin(CardPlugin)
    .run();
}
