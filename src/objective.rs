use bevy::math::IVec2;

use crate::{
    grid::{Cultivation, Grid},
    seasons::SeasonType,
};

/* I really like this too, but its unintuitive when reading
struct Objective{scoring: fn(&Grid)->u32,}*/

#[derive(Debug)]
pub struct Score(usize);

pub trait Objective {
    fn name(&self) -> &'static str;
    //fn description(&self) -> &'static str;
    fn score(&self, grid: &Grid) -> Score;
}

pub struct GameObjectives {
    objectives: [Box<dyn Objective + Send + Sync>; 4],
}
impl GameObjectives {
    pub fn objectives_for_season(&self, season: &SeasonType) -> (&dyn Objective, &dyn Objective) {
        match &season {
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
                Box::new(DuesterWald),
                Box::new(DuesterWald),
                Box::new(DuesterWald),
                Box::new(DuesterWald),
            ],
        }
    }
}

struct DuesterWald;
impl Objective for DuesterWald {
    fn name(&self) -> &'static str {
        "Duesterwald"
    }
    fn score(&self, grid: &Grid) -> Score {
        let mut count = 0;
        for field in grid.all() {
            if let Some(Cultivation::Forest) = field.cultivation {
                let surrounding =
                    vec![(1, 0).into(), (0, -1).into(), (-1, 0).into(), (0, 1).into()];
                let mut free = false;
                for offset in surrounding {
                    free = free || grid.is_free(&(field.position + offset))
                }
                if !free {
                    count += 1;
                }
            }
        }
        Score(count)
    }
}
