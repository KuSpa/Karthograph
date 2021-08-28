use bevy::prelude::*;
use kartograph_core::grid::{Cultivation, Terrain};
use std::collections::HashMap;

use crate::{CGrid, ClientGameState};

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
pub struct UIAssets {
    pub default: Handle<ColorMaterial>,
    pub highlighted: Handle<ColorMaterial>,
}

#[derive(Default)]
pub struct AssetManager {
    map: HashMap<&'static str, Handle<ColorMaterial>>,
    pub ui: UIAssets,
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

        self.font = asset_server.load("font.ttf");
        self.ui.default = materials.add(Color::SEA_GREEN.into());
        self.ui.highlighted = materials.add(Color::SALMON.into());
    }

    fn is_loaded(&self, color_mat: &Res<Assets<ColorMaterial>>, font: &Res<Assets<Font>>) -> bool {
        for (_, handle) in self.map.iter() {
            if color_mat.get(handle).is_none() {
                return false;
            }
        }
        font.get(self.font.clone()).is_some()
    }
}

pub trait AssetID {
    fn asset_id(&self) -> &'static str;
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

impl AssetID for Terrain {
    fn asset_id(&self) -> &'static str {
        match self {
            Terrain::Mountain(_) => "mountain",
            Terrain::Normal => "default",
            Terrain::Ruin => "ruin",
        }
    }
}

pub fn init_assets(
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Loading Assets");
    asset_manager.initialize(asset_server, materials);
}

fn check_readiness(
    assets: Res<AssetManager>,
    grid: Option<Res<CGrid>>,
    mut state: ResMut<State<ClientGameState>>,
    color_mat: Res<Assets<ColorMaterial>>,
    font: Res<Assets<Font>>,
) {
    if assets.is_loaded(&color_mat, &font) && grid.is_some() {
        info!("Loaded and Connected");
        info!("{:?} - Advance to Playing", state.current());
        state.set(ClientGameState::Playing).unwrap();
    }
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetManager::default())
            .add_state(ClientGameState::Loading)
            .add_startup_system(init_assets.system())
            .add_system_set(
                SystemSet::on_update(ClientGameState::Loading)
                    .with_system(check_readiness.system()),
            );
    }
}
