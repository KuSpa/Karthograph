use bevy::prelude::*;
use bevy_spicy_networking::*;
use common::card::Card;
use common::grid::{Field, Shape};
use common::network::*;
use std::net::SocketAddr;

use crate::asset_management::AssetManager;
use crate::shape::{ActiveShape, ShapeArea};
use crate::{CGrid, ClientGameState};

fn connect_to_server(mut net: ResMut<NetworkClient>) {
    let ip_addr = "127.0.0.1".parse().unwrap();
    let socket_addr = SocketAddr::new(ip_addr, 9999);
    info!("Attempt connecting to server: {:?}", socket_addr);
    net.connect(
        socket_addr,
        NetworkSettings {
            max_packet_length: 10 * 1024 * 1024,
        },
    );
}

pub struct ClientNetworkPlugin;
impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ClientPlugin)
            .add_startup_system(connect_to_server.system())
            .add_system(handle_grid_reset.system());
        client_register_network_messages(app);
    }
}

pub const CLIENT_NAME: &'static str = "Alex";

fn handle_grid_reset(
    mut com: Commands,
    assets: Res<AssetManager>,
    net: Res<NetworkClient>,
    mut network_events: EventReader<NetworkData<ConnectGrid>>,
) {
    for event in network_events.iter() {
        let grid = CGrid::new(event.grid.clone(), &mut com, &assets).unwrap();
        com.insert_resource(grid);
        warn!("Received grid. Are we sure we requested one?");
        net.send_message(ConnectPlayerName {
            player_name: CLIENT_NAME.to_string(),
        })
        .unwrap();
    }
}
