use crate::{cards::CardPile, grid::Grid};
use bevy::{prelude::*, utils::HashMap};
use bevy_spicy_networking::ConnectionId;
use common::SeasonType;
use serde::*;

mod turn;
use turn::TurnPlugin;
mod setup;
use setup::GameSetupPlugin;
mod season;
use season::SeasonPlugin;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ServerGameState {
    Setup,
    Season,
    Turn,
    Score,
    FinalScore,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(ServerGameState::Setup)
            .add_plugin(SeasonPlugin)
            .add_plugin(TurnPlugin)
            .add_plugin(GameSetupPlugin);
    }
}

//////////////////////////////////////////////////
////    Player Data
//////////////////////////////////////////////////

pub(crate) const NUM_PLAYER: usize = 1;

//// THIS IS BUT A STUB
#[derive(Default)]
pub(crate) struct Players {
    pub inner: HashMap<ConnectionId, PlayerData>,
}
impl Players {
    pub fn all_connected(&self) -> bool {
        // once the player sent back his name, we can be sure the grid was received and start the turns
        self.inner.len() == NUM_PLAYER && self.inner.values().all(|p| p.state == PlayerState::Ready)
    }

    /// creates and returns a reference to the new player
    pub fn add_player(&mut self, id: ConnectionId) -> &PlayerData {
        self.inner.insert(id.clone(), PlayerData::default());
        self.inner.get(&id).unwrap() // safe, because we just added this key
    }

    pub fn all_ready(&self) -> bool {
        self.inner.values().all(|v| v.state == PlayerState::Ready)
    }
}

#[derive(Default)]
pub struct PlayerData {
    pub(crate) state: PlayerState,
    pub(crate) grid: Grid,
    pub(crate) name: Option<String>,
}

#[derive(PartialEq, Eq)]
pub(crate) enum PlayerState {
    Pending,
    Ready,
    //Recovering
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Pending
    }
}
