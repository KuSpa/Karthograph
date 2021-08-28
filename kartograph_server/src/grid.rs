use std::{
    cmp::{min, Ordering},
    collections::{HashMap, HashSet, VecDeque},
    ops::{Add, RangeFrom},
};

use bevy::math::{IVec2, Vec2};
use derive_deref::*;
use itertools::Itertools;
use kartograph_core::grid::{Coordinate, Field, GridLike, Shape};
use kartograph_core::grid::{Cultivation, Terrain, GRID_SIZE};
use serde::{ser::SerializeTuple, Deserialize, Serialize, Serializer};

////////////////////////////////////////////////////////////////
////	Areas
////////////////////////////////////////////////////////////////
#[derive(
    Deref, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct AreaID(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaInfo {
    pub kind: Cultivation,
    pub coords: Vec<Coordinate>,
}

impl AreaInfo {
    pub fn size(&self) -> usize {
        self.coords.len()
    }
}

impl PartialOrd for AreaInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.size().partial_cmp(&other.size())
    }
}

impl Ord for AreaInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size().cmp(&other.size())
    }
}

impl From<Cultivation> for AreaInfo {
    fn from(c: Cultivation) -> Self {
        Self {
            kind: c,
            coords: Vec::default(),
        }
    }
}

////////////////////////////////////////////////////////////////
////	Cultivation
////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CultivationInformation {
    cultivation: Cultivation,
    area_id: AreaID,
}

impl CultivationInformation {
    pub fn cultivation(&self) -> &Cultivation {
        &self.cultivation
    }

    pub fn area_id(&self) -> AreaID {
        self.area_id
    }
}

impl From<Cultivation> for CultivationInformation {
    fn from(c: Cultivation) -> Self {
        Self {
            cultivation: c,
            area_id: AreaID(0),
        }
    }
}

////////////////////////////////////////////////////////////////
////	Field
////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SField {
    pub cultivation: Option<CultivationInformation>,
    pub terrain: Terrain,
    pub position: Coordinate,
}

impl SField {
    pub fn is_free(&self) -> bool {
        !self.terrain.is_mountain() && self.cultivation.is_none()
    }

    pub fn is_ruin(&self) -> bool {
        self.terrain == Terrain::Ruin
    }
}

impl From<&SField> for Field {
    fn from(f: &SField) -> Self {
        Field {
            cultivation: f.cultivation.map(|c| c.cultivation),
            terrain: f.terrain,
            position: f.position,
        }
    }
}

////////////////////////////////////////////////////////////////
////	Grid
////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Grid {
    //TODO: rename to areas
    area_infos: HashMap<AreaID, AreaInfo>,
    area_counter: RangeFrom<usize>,
    inner: [SField; GRID_SIZE * GRID_SIZE],
}

impl Grid {
    pub const SIZE: usize = GRID_SIZE as usize;

    pub fn cultivate(&mut self, shape: &Shape, position: &Coordinate) {
        for coord in shape.geometry.iter() {
            let mut field = self.at_mut(&(*coord + *position)).unwrap();
            field.cultivation = Some(shape.cultivation.into());
        }
    }

    ////////////////////////////////////////////////////////////////
    ////	Accessing
    ////////////////////////////////////////////////////////////////

    fn index(&self, coord: &Coordinate) -> Result<usize, ()> {
        if !self.is_valid_coord(coord) {
            Err(())
        } else {
            Ok((coord.x as usize) + (coord.y as usize) * Self::SIZE)
        }
    }

    pub fn mountains(&self) -> impl Iterator<Item = &SField> {
        self.all().filter(|&f| f.terrain.is_mountain())
    }

    pub fn ruins(&self) -> impl Iterator<Item = &SField> {
        self.all().filter(|&field| field.terrain == Terrain::Ruin)
    }

    pub fn neighbors(&self, coord: &Coordinate) -> impl Iterator<Item = &SField> {
        let top = *coord + (0, 1).into();
        let bottom = *coord + (0, -1).into();
        let right = *coord + (1, 0).into();
        let left = *coord + (-1, 0).into();
        vec![top, bottom, right, left]
            .into_iter()
            .filter(move |c| self.is_valid_coord(c))
            .map(move |c| self.at(&c).unwrap())
    }

    pub fn at_mut(&mut self, coord: &Coordinate) -> Result<&mut SField, ()> {
        self.index(coord).map(move |i| &mut self.inner[i])
    }

    pub fn at(&self, coord: &Coordinate) -> Result<&SField, ()> {
        self.index(coord).map(|i| &self.inner[i])
    }

    pub fn all(&self) -> impl Iterator<Item = &SField> {
        self.inner.iter()
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &SField>> {
        (0..Self::SIZE).map(move |nth| self.row(nth))
    }

    pub fn row(&self, nth: usize) -> impl Iterator<Item = &SField> {
        self.inner.iter().skip(nth * Self::SIZE).take(Self::SIZE)
    }

    pub fn columns(&self) -> impl Iterator<Item = impl Iterator<Item = &SField>> {
        (0..Self::SIZE).map(move |nth| self.column(nth))
    }

    pub fn column(&self, nth: usize) -> impl Iterator<Item = &SField> {
        self.inner.iter().skip(nth).step_by(Self::SIZE)
    }

    /// from left border to bottom
    pub fn nth_diagonal(&self, nth: usize) -> impl Iterator<Item = &SField> {
        let delta_x = min(nth, Self::SIZE - 1);
        let delta_y = if nth > (Self::SIZE - 1) {
            nth - (Self::SIZE - 1)
        } else {
            0
        }; // to avoid underflow of usize

        self.inner
            .iter()
            .skip(delta_x + delta_y * Self::SIZE)
            .step_by(Self::SIZE - 1)
            // If nth > Self::Size the iterator ends in the uppermost row, yielding no more Items
            .take(nth + 1)
    }

    /// from left border to bottom
    pub fn diagonals(&self) -> impl Iterator<Item = impl Iterator<Item = &SField>> {
        (0..(Self::SIZE * Self::SIZE - 1))
            .into_iter()
            .map(move |nth| self.nth_diagonal(nth))
    }

    ////////////////////////////////////////////////////////////////
    ////	Property testing
    ////////////////////////////////////////////////////////////////

    fn is_valid_coord(&self, coord: &Coordinate) -> bool {
        !(coord.x < 0
            || coord.x >= Self::SIZE as i32
            || coord.y < 0
            || coord.y >= Self::SIZE as i32)
    }

    pub fn is_ruin(&self, coord: &Coordinate) -> bool {
        if let Ok(index) = self.index(coord) {
            self.inner[index].terrain == Terrain::Ruin
        } else {
            false
        }
    }

    pub fn is_free(&self, coord: &Coordinate) -> bool {
        if let Ok(index) = self.index(coord) {
            self.inner[index].is_free()
        } else {
            false
        }
    }

    ////////////////////////////////////////////////////////////////
    ////	Convenience
    ////////////////////////////////////////////////////////////////

    pub fn raw_grid(&self) -> Vec<Field> {
        self.inner.iter().map(|f| f.into()).collect::<Vec<_>>()
    }

    fn next_area_id(&mut self) -> AreaID {
        AreaID(self.area_counter.next().unwrap())
    }

    pub fn mountain_coins(&mut self) -> Vec<Coordinate> {
        let mut result = Vec::default();

        for coord in self
            .mountains()
            .map(|mountain| mountain.position)
            .collect::<Vec<_>>()
        {
            if self.neighbors(&coord).any(|n| n.is_free()) {
                continue;
            }

            let field = self.at_mut(&coord).unwrap();
            if let Terrain::Mountain(ref mut coin @ true) = field.terrain {
                *coin = false;
                result.push(coord);
            }
        }

        result
    }

    fn propagate_id(&mut self, coord: &Coordinate, id: AreaID, cultivation: &Cultivation) {
        let mut queue = VecDeque::new();
        let mut fields = Vec::new();
        queue.push_front(*coord);
        while let Some(pos) = queue.pop_back() {
            fields.push(pos);

            // non valid `pos` are not added to the queue
            self.at_mut(&pos)
                .as_mut()
                .unwrap()
                .cultivation
                .as_mut()
                .unwrap()
                .area_id = id;

            let mut area_ids_to_remove: HashSet<AreaID> = HashSet::default(); // we are immutable iterating over neighbors, we cannot remove the AreaIDs on the fly
            for field in self.neighbors(&pos) {
                if let Some(area_info) = field.cultivation {
                    if area_info.area_id() < id && area_info.cultivation == *cultivation {
                        queue.push_front(field.position);
                        // the old id has been flooded, time to delete if from known id's (if not happened in earlier iteration)
                        area_ids_to_remove.insert(area_info.area_id());
                    }
                }
            }

            for id in area_ids_to_remove.iter() {
                self.area_infos.remove_entry(id);
            }
        }

        fields.sort();
        fields.dedup();
        self.area_infos.insert(
            id,
            AreaInfo {
                kind: *cultivation,
                coords: fields,
            },
        );
    }

    /// returns ids of components sorted by size (biggest first)
    pub fn area_ids(&self, cultivation: Cultivation) -> impl Iterator<Item = (&AreaID, &AreaInfo)> {
        (&self.area_infos)
            .iter()
            .filter(move |&(_, info)| info.kind == cultivation)
            .sorted_by(|lhs, rhs| lhs.1.cmp(rhs.1))
            .rev()
    }

    pub fn area_neighbors(&self, id: &AreaID) -> impl Iterator<Item = &SField> {
        self.area_infos
            .get(id)
            .unwrap()
            .coords
            .iter()
            .map(move |&field| self.neighbors(&field))
            .flatten()
            .dedup()
    }

    fn initialize(ruins: &[Coordinate], mountains: &[Coordinate]) -> Self {
        let mut grid = Grid {
            area_infos: Default::default(),
            area_counter: 1..,
            inner: [SField::default(); GRID_SIZE * GRID_SIZE],
        };
        for y in 0..Self::SIZE {
            for x in 0..Self::SIZE {
                let coord = (x, y).into();
                grid.at_mut(&coord).unwrap().position = coord;
            }
        }

        for pos in mountains.iter() {
            grid.inner[grid.index(pos).unwrap()].terrain = Terrain::Mountain(true);
        }

        for pos in ruins.iter() {
            grid.inner[grid.index(pos).unwrap()].terrain = Terrain::Ruin;
        }
        grid
    }
}

impl GridLike for Grid {
    fn is_free(&self, c: &Coordinate) -> bool {
        if let Ok(index) = self.index(c) {
            self.inner[index].is_free()
        } else {
            false
        }
    }

    fn is_ruin(&self, c: &Coordinate) -> bool {
        if let Ok(index) = self.index(c) {
            self.inner[index].is_ruin()
        } else {
            false
        }
    }
}

impl Default for Grid {
    fn default() -> Grid {
        let mountains: Vec<Coordinate> = vec![
            (2, 2).into(),
            (3, 9).into(),
            (5, 5).into(),
            (7, 1).into(),
            (8, 8).into(),
        ]; //(x,y)
        let ruins: Vec<Coordinate> = vec![
            (1, 2).into(),
            (1, 8).into(),
            (5, 1).into(),
            (5, 9).into(),
            (9, 2).into(),
            (9, 8).into(),
        ];
        Grid::initialize(&ruins, &mountains)
    }
}
