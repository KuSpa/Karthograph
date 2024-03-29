use bevy::{prelude::*, text::Text};

use crate::asset_management::AssetManager;
use crate::{
    grid::Grid,
    objective::{GameObjectives, SeasonScore},
    ui::SeasonUiMarker,
    GameState,
};

#[derive(Default, Debug)]
pub struct Season {
    season_type: SeasonType,
    passed_time: i32,
}

impl Season {
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

    pub fn season_type(&self) -> &SeasonType {
        &self.season_type
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SeasonType {
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

    pub fn marker(&self) -> SeasonMarker {
        match &self {
            Self::Spring => SeasonMarker::Spring,
            Self::Summer => SeasonMarker::Summer,
            Self::Autumn => SeasonMarker::Autumn,
            Self::Winter => SeasonMarker::Winter,
        }
    }
}

impl Default for SeasonType {
    fn default() -> Self {
        Self::Spring
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SeasonMarker {
    Spring,
    Summer,
    Autumn,
    Winter,
}

pub fn score_season(
    mut state: ResMut<State<GameState>>,
    mut ui_query: Query<(&mut Text, &SeasonMarker)>,
    season: Res<Season>,
    mut objectives: ResMut<GameObjectives>,
    grid: Res<Grid>,
) {
    let SeasonScore {
        a: first,
        b: second,
        coin_count,
    } = objectives.score_season(season.season_type(), &grid);

    // fetch season UI
    ui_query
        .iter_mut()
        .filter(|&(_, marker)| marker == &season.season_type().marker())
        .for_each(|(mut t, _)| {
            if t.sections[0].value == first.0 {
                t.sections[1].value = first.1.to_string();
            };
            if t.sections[0].value == second.0 {
                t.sections[1].value = second.1.to_string();
            };
            if t.sections[0].value == "Coins" {
                t.sections[1].value = coin_count.to_string();
            }
        });

    println!("{:?} scored {:?}", first.0, first.1);
    println!("{:?} scored {:?}", second.0, second.1);
    println!("{:?} coins were collected", coin_count);
    // TODO: coin ui
    state.pop().unwrap();
}

pub fn advance_season(
    assets: Res<AssetManager>,
    mut query: Query<(&SeasonMarker, &mut Handle<ColorMaterial>), With<SeasonUiMarker>>,
    mut season: ResMut<Season>,
    mut state: ResMut<State<GameState>>,
) {
    for (marker, mut handle) in query.iter_mut() {
        if *marker == season.season_type().marker() {
            // reset old season
            *handle = assets.ui.default.clone();
        }
    }

    if let Some(next_season) = season.next() {
        *season = next_season;
    } else {
        state.overwrite_set(GameState::End).unwrap();
        panic!("GAME SHOULD END DON'T YOU THINK?")
    }

    for (marker, mut handle) in query.iter_mut() {
        if *marker == season.season_type().marker() {
            // set new season
            *handle = assets.ui.highlighted.clone();
        }
    }
}
