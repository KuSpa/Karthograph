use crate::{
    card::{Card, Choice, Rotation},
    grid::{Coordinate, Cultivation, Field, Geometry},
};
use bevy::prelude::*;
use bevy_spicy_networking::*;
use serde::*;
use typetag::serde;

///////////////////////////////////////////
//      C->S Messages
///////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectPlayerName {
    pub player_name: String,
}
#[serde]
impl NetworkMessage for ConnectPlayerName {}
impl ServerMessage for ConnectPlayerName {
    const NAME: &'static str = "Core::ConnectName";
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CCommand {
    Place {
        choice: Choice,
        rotation: Rotation,
        mirror: bool,
        position: Coordinate,
    },
    RuinACK,
    RequestReset,
}
#[serde]
impl NetworkMessage for CCommand {}
impl ServerMessage for CCommand {
    const NAME: &'static str = "Core::CCommand";
}

pub fn server_register_network_messages(app: &mut App) {
    app.listen_for_server_message::<ConnectPlayerName>();
    app.listen_for_server_message::<CCommand>();
}

///////////////////////////////////////////
//      S->C Messages
///////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectGrid {
    pub grid: Vec<Field>, // TODO finish const-generic pull request for serde for arbitrary large arrays
}

#[serde]
impl NetworkMessage for ConnectGrid {}
impl ClientMessage for ConnectGrid {
    const NAME: &'static str = "Core::ConnectGrid";
}

#[serde]
impl NetworkMessage for Card {}

impl ClientMessage for Card {
    const NAME: &'static str = "Core::Card";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CultivationCommand {
    pub id: usize,
    pub player: String,
    pub geometry: Geometry,
    pub cultivation: Cultivation,
    pub position: Coordinate,
}

#[serde]
impl NetworkMessage for CultivationCommand {}

impl ClientMessage for CultivationCommand {
    const NAME: &'static str = "Core::CultivationCommand";
}

pub fn client_register_network_messages(app: &mut App) {
    app.listen_for_client_message::<Card>();
    app.listen_for_client_message::<ConnectGrid>();
    app.listen_for_client_message::<CultivationCommand>();
}
