use std::ops::AddAssign;

use crate::{
    asset_management::AssetID,
    grid::{Cultivation, Grid, Terrain},
    seasons::{Season, SeasonType},
};

/* I really like this too, but its unintuitive when reading
struct Objective{scoring: fn(&Grid)->u32,}*/

#[derive(Debug, Default)]
pub struct Score(usize);
impl AddAssign<usize> for Score {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

pub trait Objective: AssetID {
    fn name(&self) -> &'static str;
    //fn description(&self) -> &'static str;
    fn score(&self, grid: &Grid) -> Score;
}

pub struct GameObjectives {
    objectives: [Box<dyn Objective + Send + Sync>; 4],
}
impl GameObjectives {
    pub fn objectives_for_season(&self, season: &Season) -> (&dyn Objective, &dyn Objective) {
        match &season.season_type() {
            SeasonType::Spring => (&*self.objectives[0], &*self.objectives[1]),
            SeasonType::Summer => (&*self.objectives[1], &*self.objectives[2]),
            SeasonType::Autumn => (&*self.objectives[2], &*self.objectives[3]),
            SeasonType::Winter => (&*self.objectives[3], &*self.objectives[0]),
        }
    }
}

impl Default for GameObjectives {
    fn default() -> Self {
        Self {
            objectives: [
                Box::new(LongRoad),
                Box::new(TalDerMagier),
                Box::new(DuesterWald),
                Box::new(DuesterWald),
            ],
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
                let surrounding =
                    vec![(1, 0).into(), (0, -1).into(), (-1, 0).into(), (0, 1).into()];
                let mut free = false;
                for offset in surrounding {
                    free = free || grid.is_free(&(field.position() + offset))
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

        for field in grid
            .all()
            .filter(|field| field.terrain() == Terrain::Mountain)
        {
            for neighbor in grid.neighbor_indices(&field.position()) {
                match grid
                    .at(&neighbor)
                    .unwrap()
                    .cultivation
                    .as_ref()
                    .map(|info| info.cultivation())
                {
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
