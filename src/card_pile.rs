use crate::{asset_management::AssetManager, card::Card};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use rand::{prelude::SliceRandom, thread_rng};
use serde::Deserialize;

#[derive(Deserialize, TypeUuid, Clone)]
#[uuid = "60f975dc-d667-11eb-b8bc-0242ac130003"]
pub struct CardPile {
    pub cards: Vec<Card>,
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

pub fn cycle_cards(
    mut com: Commands,
    active_card: Query<&Card>,
    mut card_pile: Query<&mut CardPile>,
    assets: Res<AssetManager>,
) {
    if let Ok(mut pile) = card_pile.single_mut() {
        if active_card.iter().len() == 1 {
            return; // already a card active
        } else {
            if let Some(card) = pile.cards.pop() {
                card.spawn(&mut com, &assets);
            }
        }
    }
}

pub fn initialize_cards(
    // runs always
    mut com: Commands,
    query: Query<&CardPile>,
    assets: Res<AssetManager>,
    mut storage: ResMut<Assets<CardPile>>,
) {
    if let Some(cards) = storage.get_mut(&assets.cards) {
        if query.iter().len() == 0 {
            let mut pile = cards.clone();
            pile.cards.shuffle(&mut thread_rng());
            //FIXME
            // TODO: make me visible... (otherwise I rly could have used a resource lol)
            com.spawn().insert(pile);
        }
    }
}
