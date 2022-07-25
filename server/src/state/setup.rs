use std::net::SocketAddr;

use crate::cards::{CardPile, DefaultCardHandle};
use crate::state::ServerGameState;
use crate::state::{PlayerState, Players};
use bevy::prelude::*;
use bevy_spicy_networking::{NetworkData, NetworkServer, ServerNetworkEvent, ServerPlugin};
use common::network::{server_register_network_messages, ConnectGrid, ConnectPlayerName};
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NetworkPlugin).add_system_set(
            SystemSet::on_update(ServerGameState::Setup).with_system(start_game.system()),
        );
    }
}

fn start_game(
    mut state: ResMut<State<ServerGameState>>,
    handle: Res<DefaultCardHandle>,
    storage: Res<Assets<CardPile>>,
    players: Res<Players>,
) {
    if players.all_connected() && storage.get(&handle.0).is_some() {
        info!("All player connected, advance to first turn");
        state
            .set(ServerGameState::Season)
            .expect("failed to advance from Setup State");
    }
}

struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ServerPlugin::default())
            .add_startup_system(setup_networking.system())
            .add_system(handle_connection_events.system())
            .add_system(handle_name_events.system());

        server_register_network_messages(app);
    }
}

fn setup_networking(mut net: ResMut<NetworkServer>) {
    let ip_addr = "0.0.0.0".parse().unwrap();
    let socket_addr = SocketAddr::new(ip_addr, 9999);
    match net.listen(socket_addr) {
        Ok(_) => info!("Started Socket"),
        Err(err) => {
            error!("Could not start listening: {}", err);
            panic!();
        }
    };
}

fn handle_connection_events(
    net: Res<NetworkServer>,
    mut network_events: EventReader<ServerNetworkEvent>,
    mut players: ResMut<Players>,
) {
    // TODO Ignore if all players are already connected
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(id) = event {
            let player = players.add_player(id.clone());
            net.send_message(
                id.clone(),
                ConnectGrid {
                    grid: player.grid.raw_grid(),
                },
            )
            .unwrap();
        }
    }
}

fn handle_name_events(
    mut network_events: EventReader<NetworkData<ConnectPlayerName>>,
    mut players: ResMut<Players>,
) {
    for event in network_events.iter() {
        info!("Player `{:?}` connected", event.player_name);
        let player = players.inner.get_mut(&event.source()).unwrap(); // we know that we already established a connection
        player.name = Some(event.player_name.clone());
        player.state = PlayerState::Ready;
    }
}
