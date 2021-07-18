use crate::{
    asset_management::AssetManager,
    card::{Card, RuinIndicator},
    grid::Grid,
    seasons::Season,
    GameState,
};
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

impl CardPile {
    fn shuffle(&mut self) {
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

pub struct NewCard;

pub fn next_card(
    mut com: Commands,
    mut reader: EventReader<NewCard>,
    grid: Res<Grid>,
    mut current_season: ResMut<Season>,
    mut pile: ResMut<CardPile>,
    mut ruin: ResMut<RuinIndicator>,
    mut state: ResMut<State<GameState>>,
    assets: Res<AssetManager>,
) {
    // we don't care how often, just that someone wants to spawn a new card...
    if reader.iter().count() > 0 {
        if !current_season.has_time_left() {
            //trigger season end stuffy buffy flingy bingy
            state.push(GameState::SeasonScoreState).unwrap();
            return;
        }
        if let Some(mut card) = pile.cards.pop() {
            // time is added before cards are placed
            current_season.pass_time(card.time());
            // test whether you can play this card
            if !card.is_placable(&grid, &ruin) {
                println!("Card cannot be placed, fallback to default splinter card");
                card = Card::default();
                ruin.reset(); // if card is replaced, it does not need to be placed on ruins
            }

            card.spawn(&mut com, &assets, &ruin);
            ruin.reset();
        }
    }
}

pub fn initialize_cards(
    mut com: Commands,
    assets: Res<AssetManager>,
    mut storage: ResMut<Assets<CardPile>>,
    mut next: EventWriter<NewCard>,
) {
    let mut cards = storage.get_mut(&assets.cards).unwrap().clone();
    cards.shuffle();

    // will override old CardPile if existent
    com.insert_resource(cards);
    next.send(NewCard);
}
