use bevy::prelude::{ResMut, State};

use crate::GameState;

#[derive(Default, Debug)]
pub struct Season {
    season_type: SeasonType,
    passed_time: i32,
}

impl Season {
    pub fn time(&self) -> i32 {
        self.passed_time
    }

    pub fn pass_time(&mut self, time: i32) {
        self.passed_time += time;
    }

    pub fn next(&self) -> Option<Self> {
        self.season_type.next().map(|season_type| Self {
            passed_time: 0,
            season_type,
        })
    }

    pub fn has_time_left(&self) -> bool {
        self.passed_time < self.season_type.time()
    }
}
#[derive(Debug)]
enum SeasonType {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl SeasonType {
    fn time(&self) -> i32 {
        match &self {
            Self::Spring => 8,
            Self::Summer => 8,
            Self::Autumn => 7,
            Self::Winter => 6,
        }
    }

    fn next(&self) -> Option<Self> {
        match &self {
            Self::Spring => Some(Self::Summer),
            Self::Summer => Some(Self::Autumn),
            Self::Autumn => Some(Self::Winter),
            Self::Winter => None,
        }
    }
}

impl Default for SeasonType {
    fn default() -> Self {
        Self::Spring
    }
}

pub fn score_season(mut state: ResMut<State<GameState>>) {
    /* TODO do */
    state.pop().unwrap();
}

pub fn end_scoring(mut season: ResMut<Season>, mut state: ResMut<State<GameState>>) {
    if let Some(next_season) = season.next() {
        *season = next_season;
    } else {
        state.overwrite_set(GameState::End).unwrap();
        panic!("GAME SHOULD END DON'T YOU THINK?")
    }
}
