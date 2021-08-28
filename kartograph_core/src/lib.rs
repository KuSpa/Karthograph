use bevy::{ecs::{component::Component, schedule::SystemDescriptor, system::FunctionSystem}, prelude::{App, System, SystemSet}};

pub mod card;
pub mod grid;
pub mod network;
pub mod util;

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum SeasonType {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl SeasonType {
    pub fn time(&self) -> i32 {
        match &self {
            Self::Spring => 8,
            Self::Summer => 8,
            Self::Autumn => 7,
            Self::Winter => 6,
        }
    }

    pub fn next(&self) -> Option<Self> {
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
