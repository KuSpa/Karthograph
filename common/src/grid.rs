use std::{
    cmp::{min, Ordering},
    collections::{HashMap, HashSet, VecDeque},
    ops::{Add, RangeFrom},
};

use crate::{card::Rotation, util::min_f};
use bevy::{
    math::{IVec2, Vec2, Vec3},
    prelude::Transform,
};
use derive_deref::*;
use itertools::Itertools;
use serde::{ser::SerializeTuple, Deserialize, Serialize, Serializer};

#[derive(
    Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Deref, DerefMut, Hash,
)]
pub struct Coordinate(IVec2);

impl Coordinate {
    pub fn inner_copy(&self) -> IVec2 {
        self.0
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.x.partial_cmp(&other.x) {
            Some(Ordering::Equal) => self.y.partial_cmp(&other.y),
            o => o,
        }
    }
}

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

#[derive(Clone, Deserialize, Serialize, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct Geometry(Vec<Coordinate>);
impl Geometry {
    pub fn rotate_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp().perp().perp().into();
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp().into();
        }
    }

    pub fn mirror(&mut self) {
        for position in self.iter_mut() {
            position.x = -position.x;
        }
    }

    pub fn as_transforms_centered(&self, distance: f32, z: f32) -> Vec<Transform> {
        let offset = self.center_offset();
        let mut transforms = self.as_transforms(distance, z);
        transforms
            .iter_mut()
            .for_each(|trans| trans.translation -= offset * distance);

        transforms
    }

    pub fn as_transforms(&self, distance: f32, z: f32) -> Vec<Transform> {
        self.iter()
            .map(|pos| Transform::from_xyz(distance * pos.x as f32, distance * pos.y as f32, z))
            .collect()
    }

    fn min_max(&self) -> (IVec2, IVec2) {
        let mut max_v = IVec2::ZERO;
        let mut min_v = IVec2::ZERO;
        for coord in self.iter() {
            min_v = min_v.min(coord.inner_copy());
            max_v = max_v.max(coord.inner_copy());
        }
        (min_v, max_v)
    }

    pub fn max_size_in_rect(&self, size: Vec2) -> f32 {
        let (min_v, max_v) = self.min_max();
        let diff = max_v - min_v + IVec2::ONE;
        //calculate the largest possible square size
        min_f(size.x / (diff.x as f32), size.y / (diff.y as f32))
    }

    fn center_offset(&self) -> Vec3 {
        let (min_v, max_v) = self.min_max();
        let offset = max_v.as_f32() - (max_v.as_f32() - min_v.as_f32()) / 2.;
        Vec3::new(offset.x as f32, offset.y as f32, 0.)
    }
}

impl Default for Geometry {
    //Geometries are non empty
    fn default() -> Self {
        Self(vec![Coordinate::default()])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Cultivation {
    Village,
    Water,
    Farm,
    Forest,
    Goblin,
}

impl Cultivation {
    pub fn all() -> Vec<Cultivation> {
        vec![
            Self::Village,
            Self::Water,
            Self::Farm,
            Self::Forest,
            Self::Goblin,
        ]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

pub const GRID_SIZE: usize = 11;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Field {
    pub cultivation: Option<Cultivation>,
    pub terrain: Terrain,
    pub position: Coordinate,
}

impl Field {
    pub fn is_free(&self) -> bool {
        !self.terrain.is_mountain() && self.cultivation.is_none()
    }

    pub fn is_ruin(&self) -> bool {
        self.terrain == Terrain::Ruin
    }
}

pub struct Shape {
    pub coin: bool,
    pub geometry: Geometry,
    pub cultivation: Cultivation,
    pub ruin: RuinIndicator,
}

impl Shape {
    pub fn from_cultivation(c: Cultivation) -> Self {
        Shape {
            cultivation: c,
            coin: false,
            geometry: Geometry::default(),
            ruin: RuinIndicator::default(),
        }
    }

    pub fn new(g: Geometry, cult: Cultivation, ruin: RuinIndicator, coin: bool) -> Self {
        Self {
            geometry: g,
            cultivation: cult,
            ruin: ruin,
            coin,
        }
    }

    pub fn configure(&mut self, rotation: &Rotation, mirror: &bool) {
        if *mirror {
            self.mirror();
        }
        let mut own_rotation = Rotation::default();
        while *rotation != own_rotation {
            self.rotate_ck();
            own_rotation.rotate_cw();
        }
    }

    pub fn rotate_ck(&mut self) {
        for position in self.geometry.iter_mut() {
            *position = position.perp().perp().perp().into();
        }
    }

    pub fn rotate_cck(&mut self) {
        for position in self.geometry.iter_mut() {
            *position = position.perp().into();
        }
    }

    pub fn mirror(&mut self) {
        self.geometry.mirror();
    }
}

#[derive(Default, Deref, Clone, Copy)]
pub struct RuinIndicator(bool);

impl RuinIndicator {
    pub fn set(&mut self) {
        self.0 = true;
    }

    pub fn reset(&mut self) {
        self.0 = false;
    }
}

impl From<bool> for RuinIndicator {
    fn from(inner: bool) -> Self {
        Self(inner)
    }
}

pub trait GridLike {
    fn is_free(&self, c: &Coordinate) -> bool;
    fn is_ruin(&self, c: &Coordinate) -> bool;

    const SIZE: usize = GRID_SIZE;

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

    fn accepts_geometry_at(
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
}
