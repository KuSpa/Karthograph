use std::mem;
use std::ops::Index;

use crate::grid::{Coordinate, Cultivation, Geometry, RuinIndicator, Shape};
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::{log, prelude::*};
use derive_deref::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}

impl Rotation {
    pub fn rotate_cw(&mut self) {
        match &self {
            Self::North => mem::replace(self, Self::East),
            Self::East => mem::replace(self, Self::South),
            Self::South => mem::replace(self, Self::West),
            Self::West => mem::replace(self, Self::North),
        };
    }

    pub fn rotate_ccw(&mut self) {
        match &self {
            Self::North => mem::replace(self, Self::West),
            Self::East => mem::replace(self, Self::North),
            Self::South => mem::replace(self, Self::East),
            Self::West => mem::replace(self, Self::South),
        };
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::North
    }
}

#[derive(Deserialize, Clone, Debug, Serialize, PartialEq, Eq)]
pub enum Card {
    Splinter,
    Shape {
        left: Geometry,
        right: Geometry,
        cultivation: Cultivation,
    },
    Cultivation {
        geometry: Geometry,
        left: Cultivation,
        right: Cultivation,
    },
    Ruin,
}

impl Card {
    pub fn shape(&self, ch: Choice) -> Result<Shape, &'static str> {
        match (ch, self) {
            (Choice::Left, Card::Cultivation { geometry, left, .. }) => Ok(Shape::new(
                geometry.clone(),
                *left,
                RuinIndicator::default(),
                false,
            )),
            (
                Choice::Left,
                Card::Shape {
                    cultivation, left, ..
                },
            ) => Ok(Shape::new(
                left.clone(),
                *cultivation,
                RuinIndicator::default(),
                true,
            )),
            (
                Choice::Right,
                Card::Cultivation {
                    geometry, right, ..
                },
            ) => Ok(Shape::new(
                geometry.clone(),
                *right,
                RuinIndicator::default(),
                false,
            )),
            (
                Choice::Right,
                Card::Shape {
                    cultivation, right, ..
                },
            ) => Ok(Shape::new(
                right.clone(),
                *cultivation,
                RuinIndicator::default(),
                false,
            )),
            (Choice::Splinter(cu), Card::Splinter) => Ok(Shape::from_cultivation(cu)),
            _ => Err("wrong choice"),
        }
    }

    pub fn available_choices(&self) -> Vec<Choice> {
        match &self {
            Self::Ruin => Vec::default(),
            Self::Splinter => Cultivation::all()
                .iter()
                .map(|c| Choice::Splinter(*c))
                .collect(),
            _ => vec![Choice::Left, Choice::Right],
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Choice {
    Left,
    Right,
    Splinter(Cultivation),
}
