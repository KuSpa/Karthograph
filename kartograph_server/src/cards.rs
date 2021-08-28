use bevy::{
    asset::{AssetLoader, AssetPlugin, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use derive_deref::*;
use kartograph_core::card::Card;
use rand::{prelude::SliceRandom, thread_rng};
use serde::Deserialize;

#[derive(Deserialize, TypeUuid, Clone, Debug)]
#[uuid = "60f975dc-d667-11eb-b8bc-0242ac130003"]
pub struct CardPile {
    pub cards: Vec<Card>,
}

impl CardPile {
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng())
    }
}

#[derive(Default)]
pub struct CardPileLoader;

impl AssetLoader for CardPileLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let custom_asset = ron::de::from_bytes::<CardPile>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["cardpile"]
    }
}

impl Default for CardPile {
    fn default() -> Self {
        CardPile {
            cards: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct PlayPile(Vec<Card>);
#[derive(Default, Deref)]
pub struct DefaultCardHandle(pub Handle<CardPile>);

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AssetPlugin)
            .add_asset::<CardPile>()
            .init_asset_loader::<CardPileLoader>()
            .insert_resource(DefaultCardHandle::default())
            .add_startup_system(load_cards.system());
    }
}

fn load_cards(assets: Res<AssetServer>, mut handle: ResMut<DefaultCardHandle>) {
    *handle = DefaultCardHandle(assets.load("content.cardpile"));
}
