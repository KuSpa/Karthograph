use crate::asset_management::{AssetID, AssetManager};
use crate::card::RuinIndicator;
use crate::shape::{Geometry, Shape};
use crate::util::to_array;
use bevy::math::i32;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use derive_deref::*;
use itertools::Itertools;
use serde::Deserialize;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;
use std::ops::{Add, RangeFrom};

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: usize = 11;
//x=y offset
const GRID_OFFSET: f32 = SPRITE_SIZE;
#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Clone, Copy, Deserialize, Deref, DerefMut, Hash,
)]
pub struct Coordinate(IVec2);

impl Coordinate {
    pub fn inner_copy(&self) -> IVec2 {
        self.0
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl From<(usize, usize)> for Coordinate {
    fn from((x, y): (usize, usize)) -> Self {
        Self(IVec2::new(x as i32, y as i32))
    }
}

impl From<(i32, i32)> for Coordinate {
    fn from((x, y): (i32, i32)) -> Self {
        Self(IVec2::new(x, y))
    }
}

impl From<IVec2> for Coordinate {
    fn from(val: IVec2) -> Self {
        Self(val)
    }
}

impl From<Vec2> for Coordinate {
    fn from(val: Vec2) -> Self {
        Self(val.as_i32())
    }
}

impl Add<Coordinate> for Coordinate {
    type Output = Coordinate;
    fn add(self, rhs: Coordinate) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cultivation {
    //Add a None type??
    Village,
    Water,
    Farm,
    Forest,
    Goblin,
}

impl AssetID for Cultivation {
    fn asset_id(&self) -> &'static str {
        match self {
            Cultivation::Village => "village",
            Cultivation::Water => "water",
            Cultivation::Forest => "forest",
            Cultivation::Farm => "farm",
            Cultivation::Goblin => "goblin",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Terrain {
    Normal,
    Mountain(bool), // stores whether coin is already collected
    Ruin,
}

impl Terrain {
    pub fn is_mountain(&self) -> bool {
        matches!(self, Self::Mountain(_))
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain::Normal
    }
}

impl AssetID for Terrain {
    fn asset_id(&self) -> &'static str {
        match self {
            Terrain::Mountain(_) => "mountain",
            Terrain::Normal => "default",
            Terrain::Ruin => "ruin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    pub cultivation: Option<CultivationInformation>,
    terrain: Terrain,
    entity: Entity,
    position: Coordinate,
}
struct FieldComponent;

impl Field {
    fn new(entity: Entity, position: Coordinate) -> Self {
        Field {
            entity,
            terrain: Terrain::default(),
            cultivation: Option::default(),
            position,
        }
    }

    pub fn position(&self) -> Coordinate {
        self.position
    }

    pub fn terrain(&self) -> Terrain {
        self.terrain
    }

    pub fn is_free(&self) -> bool {
        !self.terrain.is_mountain() && self.cultivation.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaInfo {
    pub kind: Cultivation,
    pub field_coords: Vec<Coordinate>,
}

impl AreaInfo {
    pub fn size(&self) -> usize {
        self.field_coords.len()
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
            field_coords: Vec::default(),
        }
    }
}

#[derive(Deref, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AreaID(usize);

#[derive(Debug)]
pub struct Grid {
    entity: Entity,
    area_infos: HashMap<AreaID, AreaInfo>,
    area_counter: RangeFrom<usize>,
    inner: [Field; Grid::SIZE * Grid::SIZE],
}

impl Grid {
    pub const SIZE: usize = GRID_SIZE as usize;

    pub fn screen_to_grid(mut position: Vec2) -> Coordinate {
        position.x -= GRID_OFFSET;
        position.y -= GRID_OFFSET;
        position /= SPRITE_SIZE;
        position = position.round();
        position.into()
    }

    pub fn grid_to_screen(coord: Coordinate) -> Vec2 {
        let mut position = Vec2::new(GRID_OFFSET, GRID_OFFSET);
        position.x += coord.x as f32 * SPRITE_SIZE;
        position.y += coord.y as f32 * SPRITE_SIZE;
        position
    }

    fn is_valid_coord(&self, coord: &Coordinate) -> bool {
        !(coord.x < 0
            || coord.x >= Self::SIZE as i32
            || coord.y < 0
            || coord.y >= Self::SIZE as i32)
    }

    fn index(&self, coord: &Coordinate) -> Result<usize, ()> {
        if !self.is_valid_coord(coord) {
            Err(())
        } else {
            Ok((coord.x as usize) + (coord.y as usize) * Self::SIZE)
        }
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

    pub fn try_cultivate(
        &mut self,
        shape: &Shape,
        coord: &Coordinate,
        assets: &AssetManager,
        mut handles: &mut Query<&mut Handle<ColorMaterial>>,
    ) -> Result<Vec<Coordinate>, &'static str> {
        if self.accepts_geometry_at(shape.geometry(), coord, &shape.ruin()) {
            self.cultivate(shape, coord, assets, &mut handles);
            Ok(shape.geometry().iter().map(|pos| *pos + *coord).collect())
        } else {
            Err("Can't place the shape here")
        }
    }

    fn next_area_id(&mut self) -> AreaID {
        AreaID(self.area_counter.next().unwrap())
    }

    pub fn mountain_coins(&mut self) -> Vec<Coordinate> {
        let mut result = Vec::default();

        for coord in self
            .mountains()
            .map(|mountain| mountain.position())
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

    fn cultivate(
        &mut self,
        shape: &Shape,
        coord: &Coordinate,
        assets: &AssetManager,
        handles: &mut Query<&mut Handle<ColorMaterial>>,
    ) {
        for position in shape.geometry().iter() {
            let mut field = self.at_mut(&(*coord + *position)).unwrap();
            field.cultivation = Some(shape.cultivation().into());
            let mut handle = handles.get_mut(field.entity).unwrap();
            *handle = assets.fetch(shape.cultivation().asset_id()).unwrap();
        }

        let id = self.next_area_id();
        self.area_infos
            .insert(id, AreaInfo::from(shape.cultivation()));
        self.propagate_id(coord, id, &shape.cultivation());
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
                        queue.push_front(field.position());
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
                field_coords: fields,
            },
        );
    }

    pub fn mountains(&self) -> impl Iterator<Item = &Field> {
        self.all().filter(|&f| f.terrain().is_mountain())
    }

    pub fn ruins(&self) -> impl Iterator<Item = &Field> {
        self.all().filter(|&field| field.terrain() == Terrain::Ruin)
    }

    pub fn neighbors(&self, coord: &Coordinate) -> impl Iterator<Item = &Field> {
        let top = *coord + (0, 1).into();
        let bottom = *coord + (0, -1).into();
        let right = *coord + (1, 0).into();
        let left = *coord + (-1, 0).into();
        vec![top, bottom, right, left]
            .into_iter()
            .filter(move |c| self.is_valid_coord(c))
            .map(move |c| self.at(&c).unwrap())
    }

    /// returns ids of components sorted by size (biggest first)
    pub fn area_ids(&self, cultivation: Cultivation) -> impl Iterator<Item = (&AreaID, &AreaInfo)> {
        (&self.area_infos)
            .iter()
            .filter(move |&(_, info)| info.kind == cultivation)
            .sorted_by(|lhs, rhs| lhs.1.cmp(rhs.1))
            .rev()
    }

    pub fn area_neighbors(&self, id: &AreaID) -> impl Iterator<Item = &Field> {
        self.area_infos
            .get(id)
            .unwrap()
            .field_coords
            .iter()
            .map(move |&field| self.neighbors(&field))
            .flatten()
            .dedup()
    }

    pub fn accepts_geometry(&self, geom: &Geometry, ruins: &RuinIndicator) -> bool {
        // little bit ugly as the grid assumes knowledge on what orientations the geometry can have, but hm
        let mut mirrored = geom.clone();
        mirrored.mirror();
        for mut geom in [geom.clone(), mirrored] {
            for _ in 0..4 {
                for field in self.all() {
                    if self.accepts_geometry_at(&geom, &field.position, ruins) {
                        return true;
                    }
                }
                geom.rotate_clockwise();
            }
        }
        false
    }

    pub fn accepts_geometry_at(
        &self,
        geom: &Geometry,
        coord: &Coordinate,
        ruins: &RuinIndicator,
    ) -> bool {
        let mut on_ruin = false;
        for &pos in geom.iter() {
            if self.is_ruin(&(pos + *coord)) {
                on_ruin = true;
            }
            if !self.is_free(&(pos + *coord)) {
                return false;
            }
        }
        // == !(ruin && !on_ruin) <= if it SHOULD be on a ruin, but is NOT, then return false
        !(**ruins) || on_ruin
    }

    fn initialize(com: &mut Commands, ruins: &[Coordinate], mountains: &[Coordinate]) -> Self {
        let mut temp_vec: Vec<Field> = Vec::default();
        for y in 0..Self::SIZE {
            for x in 0..Self::SIZE {
                let entity = com.spawn().id();
                temp_vec.push(Field::new(entity, (x, y).into()));
            }
        }
        let entity = com.spawn().id();
        let mut grid = Grid {
            entity,
            area_infos: Default::default(),
            area_counter: 1..,
            inner: to_array::<Field, { Self::SIZE * Self::SIZE }>(temp_vec),
        };

        for pos in mountains.iter() {
            grid.inner[grid.index(pos).unwrap()].terrain = Terrain::Mountain(true);
        }

        for pos in ruins.iter() {
            grid.inner[grid.index(pos).unwrap()].terrain = Terrain::Ruin;
        }
        grid
    }

    pub fn at_mut(&mut self, coord: &Coordinate) -> Result<&mut Field, ()> {
        self.index(coord).map(move |i| &mut self.inner[i])
    }

    pub fn at(&self, coord: &Coordinate) -> Result<&Field, ()> {
        self.index(coord).map(|i| &self.inner[i])
    }

    pub fn new(com: &mut Commands) -> Self {
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
        Grid::initialize(com, &ruins, &mountains)
    }

    pub fn all(&self) -> impl Iterator<Item = &Field> {
        self.inner.iter()
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Field>> {
        (0..Self::SIZE).map(move |nth| self.row(nth))
    }

    // use result for safety?
    pub fn row(&self, nth: usize) -> impl Iterator<Item = &Field> {
        self.inner.iter().skip(nth * Self::SIZE).take(Self::SIZE)
    }

    pub fn columns(&self) -> impl Iterator<Item = impl Iterator<Item = &Field>> {
        (0..Self::SIZE).map(move |nth| self.column(nth))
    }

    // use result for safety?
    pub fn column(&self, nth: usize) -> impl Iterator<Item = &Field> {
        self.inner.iter().skip(nth).step_by(Self::SIZE)
    }

    /// from left border to bottom
    pub fn nth_diagonal(&self, nth: usize) -> impl Iterator<Item = &Field> {
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
    pub fn diagonals(&self) -> impl Iterator<Item = impl Iterator<Item = &Field>> {
        (0..(Self::SIZE * Self::SIZE - 1))
            .into_iter()
            .map(move |nth| self.nth_diagonal(nth))
    }
}

pub fn init_grid(mut com: Commands, assets: Res<AssetManager>) {
    let grid = Grid::new(&mut com);

    for field in grid.all() {
        let mat = assets.fetch(field.terrain.asset_id()).unwrap();
        let pos = field.position();
        //THE FIELD OR THE GRID SHOULD DO THIS ITSELF
        com.entity(field.entity)
            .insert(FieldComponent)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                material: mat,
                transform: Transform::from_xyz(
                    pos.x as f32 * SPRITE_SIZE + GRID_OFFSET,
                    pos.y as f32 * SPRITE_SIZE + GRID_OFFSET,
                    -0.1,
                ),
                ..Default::default()
            });
    }

    com.insert_resource(grid);
}
