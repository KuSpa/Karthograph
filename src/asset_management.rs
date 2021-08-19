use bevy::prelude::*;
use std::collections::HashMap;

use crate::{card_pile::CardPile, GameState};

const ASSETS: [(&str, &str); 10] = [
    ("mountain", "mountain.png"),
    ("ruin", "ruin.png"),
    ("default", "default.png"),
    ("farm", "farm.png"),
    ("forest", "forest.png"),
    ("goblin", "goblin.png"),
    ("water", "water.png"),
    ("village", "village.png"),
    ("blank_card", "card.png"),
    ("coin", "coin.png"),
];

#[derive(Default)]
pub struct AssetManager {
    map: HashMap<&'static str, Handle<ColorMaterial>>,
    pub cards: Handle<CardPile>,
    pub font: Handle<Font>,
}
impl AssetManager {
    fn insert_asset(&mut self, name: &'static str, handle: Handle<ColorMaterial>) {
        self.map.insert(name, handle);
    }
    pub fn fetch(&self, name: &'static str) -> Option<Handle<ColorMaterial>> {
        self.map.get(name).cloned()
    }

    pub fn initialize(
        &mut self,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for (name, path) in ASSETS {
            let asset = materials.add(asset_server.load(path).clone().into());
            self.insert_asset(name, asset);
        }
        self.cards = asset_server.load("content.cardpile");
        self.font = asset_server.load("font.ttf");
    }

    fn is_loaded(
        &self,
        color_mat: &Res<Assets<ColorMaterial>>,
        card_pile: &Res<Assets<CardPile>>,
        font: &Res<Assets<Font>>,
    ) -> bool {
        for (_, handle) in self.map.iter() {
            if color_mat.get(handle).is_none() {
                return false;
            }
        }
        card_pile.get(self.cards.clone()).is_some() && font.get(self.font.clone()).is_some()
    }
}

pub trait AssetID {
    fn asset_id(&self) -> &'static str;
}

pub fn init_assets(
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    asset_manager.initialize(asset_server, materials);
}

pub fn check_readiness(
    assets: Res<AssetManager>,
    mut state: ResMut<State<GameState>>,
    color_mat: Res<Assets<ColorMaterial>>,
    card_pile: Res<Assets<CardPile>>,
    font: Res<Assets<Font>>,
) {
    if assets.is_loaded(&color_mat, &card_pile, &font) {
        state.set(GameState::SeasonState).unwrap();
    }
}
