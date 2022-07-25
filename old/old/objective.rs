use std::{
    cmp::{max, min},
    fmt::{self, Debug},
    ops::AddAssign,
};

use crate::old::{
    asset_management::AssetID,
    grid::{Coordinate, Cultivation, Grid},
    seasons::SeasonType,
};
use bevy::utils::HashMap;
use itertools::Itertools;
use rand::{prelude::SliceRandom, thread_rng};

/* I really like this too, but its unintuitive when reading
struct Objective{scoring: fn(&Grid)->u32,}*/

#[derive(Debug, Default, Clone, Copy)]
pub struct Score(usize);
impl AddAssign<usize> for Score {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

pub struct SeasonScore {
    pub a: (&'static str, Score),
    pub b: (&'static str, Score),
    pub coin_count: usize,
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            0 => write!(f, "-"),
            inner => std::fmt::Display::fmt(&inner, f),
        }
    }
}

pub trait Objective: AssetID {
    fn name(&self) -> &'static str;
    //fn description(&self) -> &'static str;
    fn score(&self, grid: &Grid) -> Score;
}

pub struct GameObjectives {
    objectives: [Box<dyn Objective + Send + Sync>; 4],
    scores: [Option<SeasonScore>; 4],
    current_coins: Vec<Vec<Coordinate>>,
}
impl GameObjectives {
    pub fn objectives_for_season(&self, season: &SeasonType) -> (&dyn Objective, &dyn Objective) {
        (
            &*self.objectives[Self::idx(season)],
            &*self.objectives[(Self::idx(season) + 1) % 4],
        )
    }

    fn idx(season: &SeasonType) -> usize {
        match &season {
            SeasonType::Spring => 0,
            SeasonType::Summer => 1,
            SeasonType::Autumn => 2,
            SeasonType::Winter => 3,
        }
    }

    pub fn add_coin(&mut self, position: Vec<Coordinate>) {
        self.current_coins.push(position);
    }

    pub fn score_season(&mut self, season: &SeasonType, grid: &Grid) -> &SeasonScore {
        let idx = Self::idx(season);
        if self.scores[idx].is_none() {
            let (first, second) = self.objectives_for_season(season);
            self.scores[idx] = Some(SeasonScore {
                a: (first.name(), first.score(grid)),
                b: (second.name(), second.score(grid)),
                coin_count: self.current_coins.len(),
            });
        }

        self.scores[idx].as_ref().unwrap()
    }
}

impl Default for GameObjectives {
    fn default() -> Self {
        let mut objectives: Vec<Box<dyn Objective + Send + Sync>> = vec![
            Box::new(PfadDesWaldes),
            Box::new(Metropole),
            Box::new(SchildDesReichs),
            Box::new(AusgedehnteStraende),
            Box::new(Gruenflaeche),
            Box::new(Grenzland),
            Box::new(GoldenCorn),
            Box::new(TalDerMagier),
            Box::new(LongRoad),
            Box::new(DuesterWald),
            Box::new(SchillerndeEbene),
            Box::new(Schildwald),
            Box::new(DieKessel),
            Box::new(UnzugaenglicheBaronie),
            Box::new(Bewaesserungskanal),
            Box::new(BastionInTheWilderness),
        ];
        objectives.shuffle(&mut thread_rng());

        Self {
            objectives: [
                objectives.pop().unwrap(),
                objectives.pop().unwrap(),
                objectives.pop().unwrap(),
                objectives.pop().unwrap(),
            ],
            scores: Default::default(),
            current_coins: Default::default(),
        }
    }
}

struct DuesterWald;

impl AssetID for DuesterWald {
    fn asset_id(&self) -> &'static str {
        "duesterwald"
    }
}

impl Objective for DuesterWald {
    fn name(&self) -> &'static str {
        "Duesterwald"
    }
    fn score(&self, grid: &Grid) -> Score {
        let mut count = 0;
        for field in grid.all() {
            if let Some(Cultivation::Forest) =
                field.cultivation.as_ref().map(|info| info.cultivation())
            {
                let mut free = false;
                for neighbor in grid.neighbors(&field.position()) {
                    free = free || neighbor.is_free()
                }
                if !free {
                    count += 1;
                }
            }
        }
        Score(count)
    }
}

struct TalDerMagier;

impl AssetID for TalDerMagier {
    fn asset_id(&self) -> &'static str {
        "tal_der_magier"
    }
}

impl Objective for TalDerMagier {
    fn name(&self) -> &'static str {
        "Tal der Magier"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();

        for field in grid.all().filter(|field| field.terrain().is_mountain()) {
            for neighbor in grid.neighbors(&field.position()) {
                match neighbor.cultivation.as_ref().map(|info| info.cultivation()) {
                    Some(Cultivation::Water) => score += 1,
                    Some(Cultivation::Farm) => score += 1,
                    _ => {}
                }
            }
        }

        score
    }
}

struct LongRoad;

impl AssetID for LongRoad {
    fn asset_id(&self) -> &'static str {
        "long_road"
    }
}

impl Objective for LongRoad {
    fn name(&self) -> &'static str {
        "Die Lange Straße"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for diagonal in grid.diagonals().take(Grid::SIZE) {
            let mut is_free = false;
            for field in diagonal {
                if field.is_free() {
                    is_free = true;
                }
            }

            if !is_free {
                score += 3;
            }
        }

        score
    }
}

struct BastionInTheWilderness;

impl AssetID for BastionInTheWilderness {
    fn asset_id(&self) -> &'static str {
        "bastion_in_the_wilderness"
    }
}

impl Objective for BastionInTheWilderness {
    fn name(&self) -> &'static str {
        "Bastion In The Wilderness"
    }

    fn score(&self, grid: &Grid) -> Score {
        Score(
            grid.area_ids(Cultivation::Village)
                .filter(|(_, info)| info.size() >= 6)
                .count()
                * 8,
        )
    }
}

struct Metropole;

impl AssetID for Metropole {
    fn asset_id(&self) -> &'static str {
        "metropole"
    }
}

impl Objective for Metropole {
    fn name(&self) -> &'static str {
        "Metropole"
    }

    fn score(&self, grid: &Grid) -> Score {
        'outer: for (&id, info) in grid.area_ids(Cultivation::Village) {
            let mut neighbors = grid.area_neighbors(&id);
            for mountain in grid.mountains() {
                if neighbors.contains(&mountain) {
                    continue 'outer;
                }
            }

            return Score(info.size());
        }
        Score::default()
    }
}

struct GoldenCorn;

impl AssetID for GoldenCorn {
    fn asset_id(&self) -> &'static str {
        "corn"
    }
}

impl Objective for GoldenCorn {
    fn name(&self) -> &'static str {
        "Goldener Kornspeicher"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for ruin in grid.ruins() {
            if ruin.cultivation.as_ref().map(|info| info.cultivation()) == Some(&Cultivation::Farm)
            {
                score += 3;
            }

            for neighbor in grid.neighbors(&ruin.position()) {
                if neighbor.cultivation.as_ref().map(|info| info.cultivation())
                    == Some(&Cultivation::Water)
                {
                    score += 1;
                }
            }
        }
        score
    }
}

struct Grenzland;

impl AssetID for Grenzland {
    fn asset_id(&self) -> &'static str {
        "grenzland"
    }
}

impl Objective for Grenzland {
    fn name(&self) -> &'static str {
        "Grenzland"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for mut iter in grid.rows() {
            if iter.all(|field| !field.is_free()) {
                score += 6;
            }
        }

        for mut col in grid.columns() {
            if col.all(|field| !field.is_free()) {
                score += 6;
            }
        }
        score
    }
}

struct Gruenflaeche;

impl AssetID for Gruenflaeche {
    fn asset_id(&self) -> &'static str {
        "gruenflaeche"
    }
}

impl Objective for Gruenflaeche {
    fn name(&self) -> &'static str {
        "Grünfläche"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for mut row in grid.rows() {
            if row.any(|field| {
                field.cultivation.as_ref().map(|f| f.cultivation()) == Some(&Cultivation::Forest)
            }) {
                score += 1;
            }
        }

        for mut col in grid.columns() {
            if col.any(|field| {
                field.cultivation.as_ref().map(|f| f.cultivation()) == Some(&Cultivation::Forest)
            }) {
                score += 1;
            }
        }
        score
    }
}

struct AusgedehnteStraende;

impl AssetID for AusgedehnteStraende {
    fn asset_id(&self) -> &'static str {
        "ausgedehnte_straende"
    }
}

impl Objective for AusgedehnteStraende {
    fn name(&self) -> &'static str {
        "Ausgedehnte Strände"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for (&id, info) in grid.area_ids(Cultivation::Water) {
            let mut neighbors = grid.area_neighbors(&id);
            if neighbors.any(|f| {
                f.cultivation.as_ref().map(|f| f.cultivation()) == Some(&Cultivation::Farm)
            }) {
                continue;
            }

            if info
                .field_coords
                .iter()
                .any(|pos| grid.neighbors(pos).count() < 4)
            {
                continue;
            }
            score += 3;
        }

        for (&id, info) in grid.area_ids(Cultivation::Farm) {
            let mut neighbors = grid.area_neighbors(&id);
            if neighbors.any(|f| {
                f.cultivation.as_ref().map(|f| f.cultivation()) == Some(&Cultivation::Water)
            }) {
                continue;
            }

            if info
                .field_coords
                .iter()
                .any(|pos| grid.neighbors(pos).count() < 4)
            {
                continue;
            }
            score += 3;
        }

        score
    }
}

struct SchildDesReichs;

impl AssetID for SchildDesReichs {
    fn asset_id(&self) -> &'static str {
        "schild_des_reichs"
    }
}

impl Objective for SchildDesReichs {
    fn name(&self) -> &'static str {
        "SchildDesReichs"
    }

    fn score(&self, grid: &Grid) -> Score {
        if let Some((_, second_largest_village)) = grid.area_ids(Cultivation::Village).nth(1) {
            Score(second_largest_village.size())
        } else {
            Score::default()
        }
    }
}

struct SchillerndeEbene;

impl AssetID for SchillerndeEbene {
    fn asset_id(&self) -> &'static str {
        "schillernde_ebene"
    }
}

impl Objective for SchillerndeEbene {
    fn name(&self) -> &'static str {
        "Schillernde Ebene"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for (id, _) in grid.area_ids(Cultivation::Village) {
            if grid
                .area_neighbors(id)
                .filter_map(|f| f.cultivation.as_ref().map(|f| f.cultivation()))
                .sorted()
                .dedup()
                .count()
                >= 3
            {
                score += 3;
            }
        }

        score
    }
}

struct UnzugaenglicheBaronie;

impl AssetID for UnzugaenglicheBaronie {
    fn asset_id(&self) -> &'static str {
        "unzugaengliche_baronie"
    }
}

impl Objective for UnzugaenglicheBaronie {
    fn name(&self) -> &'static str {
        "Unzugängliche Baronie"
    }

    fn score(&self, grid: &Grid) -> Score {
        // stores the biggest square having this field as bottom right corner
        let mut matrix = [[0; Grid::SIZE]; Grid::SIZE];
        let mut result = 0;
        for x in 0..Grid::SIZE {
            for y in 0..Grid::SIZE {
                // safe: we are within the gridsize
                if grid.at(&(x, y).into()).unwrap().is_free() {
                    continue;
                }
                let left = if x > 0 { matrix[x - 1][y] } else { 0 };
                let right = if y > 0 { matrix[x][y - 1] } else { 0 };
                let diagonal = if left > 0 && right > 0 {
                    matrix[x - 1][y - 1]
                } else {
                    0
                };

                matrix[x][y] = 1 + min(min(left, right), diagonal);
                result = max(result, matrix[x][y]);
            }
        }
        Score(result)
    }
}

struct DieKessel;

impl AssetID for DieKessel {
    fn asset_id(&self) -> &'static str {
        "die_kessel"
    }
}

impl Objective for DieKessel {
    fn name(&self) -> &'static str {
        "Die Kessel"
    }

    fn score(&self, grid: &Grid) -> Score {
        Score(
            grid.all()
                .filter(|field| {
                    field.is_free()
                        && grid
                            .neighbors(&field.position())
                            .all(|neigh| !neigh.is_free())
                })
                .count(),
        )
    }
}

struct Schildwald;

impl AssetID for Schildwald {
    fn asset_id(&self) -> &'static str {
        "schildwald"
    }
}

impl Objective for Schildwald {
    fn name(&self) -> &'static str {
        "Schildwald"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();

        // top and bottom row
        score += grid
            .row(0)
            .filter(|&field| {
                field.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Forest)
            })
            .count();

        score += grid
            .row(Grid::SIZE - 1)
            .filter(|&field| {
                field.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Forest)
            })
            .count();

        // left and right column - corners
        score += grid
            .column(0)
            .skip(1)
            .take(Grid::SIZE - 2)
            .filter(|&field| {
                field.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Forest)
            })
            .count();

        score += grid
            .column(Grid::SIZE - 1)
            .skip(1)
            .take(Grid::SIZE - 2)
            .filter(|&field| {
                field.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Forest)
            })
            .count();

        score
    }
}

struct Bewaesserungskanal;

impl AssetID for Bewaesserungskanal {
    fn asset_id(&self) -> &'static str {
        "bewaesserungskanal"
    }
}

impl Objective for Bewaesserungskanal {
    fn name(&self) -> &'static str {
        "Bewässerungskanal"
    }

    fn score(&self, grid: &Grid) -> Score {
        let mut score = Score::default();
        for field in grid.all() {
            if let Some(Cultivation::Farm) = field.cultivation.as_ref().map(|i| i.cultivation()) {
                if grid.neighbors(&field.position()).any(|f| {
                    f.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Water)
                }) {
                    score += 1;
                }
            }

            if let Some(Cultivation::Water) = field.cultivation.as_ref().map(|i| i.cultivation()) {
                if grid.neighbors(&field.position()).any(|f| {
                    f.cultivation.as_ref().map(|i| i.cultivation()) == Some(&Cultivation::Farm)
                }) {
                    score += 1;
                }
            }
        }
        score
    }
}

struct PfadDesWaldes;

impl AssetID for PfadDesWaldes {
    fn asset_id(&self) -> &'static str {
        "pfad_des_waldes"
    }
}

impl Objective for PfadDesWaldes {
    fn name(&self) -> &'static str {
        "Pfad des Waldes"
    }

    fn score(&self, grid: &Grid) -> Score {
        // union find on mountains, every forest is a union
        // however, we don't really care how the resulting structure is, just IF the mountain has been joint with others
        let mut union_find: HashMap<Coordinate, bool> =
            grid.mountains().map(|f| (f.position(), false)).collect();
        for (forest_id, _) in grid.area_ids(Cultivation::Forest) {
            let mut neighbor_mountains = grid
                .area_neighbors(forest_id)
                .filter(|f| f.terrain().is_mountain())
                // mountains can occur more than once as neighbors of an area
                // however, as soon as there are two different mountains we set all to true anyway
                // so we do not need to sort before dedup()
                .dedup();
            if let Some(first_mountain) = neighbor_mountains.next() {
                for second_mountain in neighbor_mountains {
                    // the "union"
                    union_find.insert(first_mountain.position(), true);
                    union_find.insert(second_mountain.position(), true);
                }
            }
        }
        Score(union_find.iter().filter(|&k_v| *k_v.1).count() * 3)
    }
}
