use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_spicy_networking::ServerPlugin;
use karthograph_core::*;

fn main() {
    let mut app = App::build();
    app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(
        1000 / 30,
    )))
    .add_plugins(MinimalPlugins)
    .add_plugin(ServerPlugin::default());
    //shared::server_register_network_messages(&mut app);

    app.run();
}
