use bevy::prelude::*;
use std::collections::HashMap;

use crate::card_pile::CardPile;

const ASSETS: [(&'static str, &'static str); 9] = [
    ("mountain", "mountain.png"),
    ("ruin", "ruin.png"),
    ("default", "default.png"),
    ("farm", "farm.png"),
    ("forest", "forest.png"),
    ("goblin", "goblin.png"),
    ("water", "water.png"),
    ("village", "village.png"),
    ("blank_card", "card.png"),
];

#[derive(Default)]
pub struct AssetManager {
    map: HashMap<&'static str, Handle<ColorMaterial>>,
    pub cards: Handle<CardPile>,
    pub font: Handle<Font>
}
impl AssetManager {
    fn insert_asset(&mut self, name: &'static str, handle: Handle<ColorMaterial>) {
        self.map.insert(name, handle);
    }
    pub fn fetch(&self, name: &'static str) -> Option<Handle<ColorMaterial>> {
        self.map.get(name).cloned()
    }

    pub fn new(
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let mut manager = Self::default();
        for (name, path) in ASSETS {
            let asset = materials.add(asset_server.load(path).clone().into());
            manager.insert_asset(name.into(), asset);
        }
        manager.cards = asset_server.load("content.cardpile");
        manager.font = asset_server.load("font.ttf");
        manager
    }
}

pub fn init_assets(
    mut com: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = AssetManager::new(asset_server, materials);
    com.insert_resource(assets);
}
