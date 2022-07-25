use std::ops::{RangeFrom, RangeTo};

use crate::state::season::Season;
use crate::{cards::CardPile, state::ServerGameState};
use bevy::prelude::*;
use bevy_spicy_networking::{NetworkData, NetworkServer};
use common::card::Card;
use common::grid::{GridLike, Shape};
use common::network::{CCommand, CultivationCommand};

use super::{PlayerState, Players};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]

pub struct TurnNumber(usize);

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(ServerGameState::Turn).with_system(new_card.system()),
        )
        .add_system_set(
            SystemSet::on_update(ServerGameState::Turn)
                .with_system(receive_client_commands.system())
                .with_system(end_turn.system()),
        );
    }
}

fn new_card(
    mut com: Commands,
    mut cards: ResMut<CardPile>,
    mut season: ResMut<Season>,
    net: Res<NetworkServer>,
    mut players: ResMut<Players>,
) {
    for player in players.inner.values_mut() {
        player.state = PlayerState::Pending;
    }

    let card = cards.cards.pop().unwrap();
    info!("Draw a card");
    match card {
        Card::Cultivation { .. } => season.pass_time(2),
        Card::Shape { .. } => season.pass_time(1),
        _ => (),
    };

    net.broadcast(card.clone());
    com.insert_resource(card);
}

struct Counter(RangeFrom<usize>);
impl Default for Counter {
    fn default() -> Self {
        Self(0..)
    }
}

fn receive_client_commands(
    mut counter: Local<Counter>,
    mut client_commands: EventReader<NetworkData<CCommand>>,
    mut players: ResMut<Players>,
    card_res: Option<Res<Card>>,
    net: Res<NetworkServer>,
) {
    for command in client_commands.iter() {
        if let Some(ref card) = card_res {
            let sender = command.source();
            let data = players.inner.get_mut(&sender).unwrap();
            match command as &CCommand {
                CCommand::RuinACK => data.state = PlayerState::Ready,
                CCommand::Place {
                    choice,
                    rotation,
                    position,
                    mirror,
                } => {
                    if let Ok(mut shape) = card.shape(*choice) {
                        shape.configure(rotation, mirror);
                        if data
                            .grid
                            .accepts_geometry_at(&shape.geometry, position, &shape.ruin)
                        {
                            info!("Write Shape, broadcast change");
                            data.grid.cultivate(&shape, position);
                            net.broadcast(CultivationCommand {
                                id: counter.0.next().unwrap(),
                                player: data.name.clone().unwrap(),
                                geometry: shape.geometry.clone(),
                                cultivation: shape.cultivation,
                                position: position.clone(),
                            });
                            data.state = PlayerState::Ready;

                            return;
                        }
                    }
                    error!("Player sent invalid Place Request: {:?}", &command);
                }
                CCommand::RequestReset => todo!(),
            }
        }
    }
}

fn end_turn(players: Res<Players>, mut state: ResMut<State<ServerGameState>>) {
    if players.all_ready() {
        state.pop().unwrap();
    }
}
