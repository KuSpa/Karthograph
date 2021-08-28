use crate::{
    cards::{CardPile, DefaultCardHandle},
    state::ServerGameState,
};
use bevy::prelude::*;
use kartograph_core::SeasonType;

pub struct SeasonPlugin;

impl Plugin for SeasonPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(
            SystemSet::on_exit(ServerGameState::Setup).with_system(prepare_season.system()),
        )
        .add_system_set(
            SystemSet::on_enter(ServerGameState::Season).with_system(start_season.system()),
        )
        .add_system_set(
            SystemSet::on_resume(ServerGameState::Season).with_system(advance.system()),
        );
    }
}

#[derive(Default, Debug)]
pub(crate) struct Season {
    s_type: SeasonType,
    passed_time: i32,
}

impl Season {
    pub fn pass_time(&mut self, time: i32) {
        self.passed_time += time;
    }
}

fn prepare_season(mut com: Commands) {
    com.insert_resource(Season::default())
}

fn start_season(
    mut com: Commands,
    handle: Res<DefaultCardHandle>,
    storage: Res<Assets<CardPile>>,
    state: ResMut<State<ServerGameState>>,
    season: Res<Season>,
) {
    // clone default cards
    // shuffle goblins into cardpile
    let mut cards = storage.get(&handle.0).cloned().unwrap();
    //cards.shuffle(); TODO
    com.insert_resource(cards);

    // continue like normal
    advance(state, season)
}

fn advance(mut state: ResMut<State<ServerGameState>>, season: Res<Season>) {
    if season.passed_time < season.s_type.time() {
        info!(
            "Advance to next turn. Current Seasontime : {:?} of {:?}",
            season.passed_time,
            season.s_type.time()
        );
        state.push(ServerGameState::Turn).unwrap();
    } else {
        info!(
            "Advance to Scoring -- Seasontime exceeded threshhold {:?}/{:?}",
            season.passed_time,
            season.s_type.time()
        );
        state.set(ServerGameState::Score).unwrap();
    }

    // The end of the scoring section is responsible to advance to the next season
}
