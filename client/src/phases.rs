use bevy::{ecs::{schedule::ShouldRun, system::Command}, input::mouse::MouseButtonInput, prelude::Plugin};
use bevy_spicy_networking::NetworkData;
use common::{card::{Card, Choice, Rotation}, grid::{Coordinate, Cultivation, Geometry, Shape}, network::CultivationCommand};

use crate::{
    asset_management::{AssetID, AssetManager},
    network::CLIENT_NAME,
    shape::{spawn_shape, ActiveShape, ShapePlugin},
    CGrid, ClientGameState, MousePosition, SPRITE_SIZE,
};
use bevy::prelude::*;

pub struct ActivePhasePlugin;

impl Plugin for ActivePhasePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(
            SystemSet::on_enter(ClientGameState::ActiveTurn).with_system(setup_choices.system()),
        )
        .add_plugin(ShapePlugin)
        .add_system_set(
            SystemSet::new().with_run_criteria(play_state)
                .with_system(handle_card.system())
                .with_system(handle_cultivation.system()),
        );
    }
}

fn setup_choices(mut com: Commands, assets: Res<AssetManager>, card: Res<Card>) {
    let parent_transform = Transform::from_xyz(1200., 500., 0.);
    let children = match &*card {
        Card::Ruin => setup_ruin(&mut com, &assets),
        Card::Splinter => setup_splinter(&card, &mut com, &assets),
        _ => setup_normal(&card, &mut com, &assets),
    };
    com.spawn()
        .insert(GlobalTransform::default())
        .insert(parent_transform)
        .push_children(&children);
}

fn setup_splinter(card: &Card, mut com: &mut Commands, assets: &Res<AssetManager>) -> Vec<Entity> {
    const SPLINTER_OFFSET: f32 = 75.;
    let mut transforms = vec![
        Transform::from_xyz(SPLINTER_OFFSET, SPLINTER_OFFSET, 0.1),
        Transform::from_xyz(SPLINTER_OFFSET, -SPLINTER_OFFSET, 0.1),
        Transform::from_xyz(-SPLINTER_OFFSET, SPLINTER_OFFSET, 0.1),
        Transform::from_xyz(-SPLINTER_OFFSET, -SPLINTER_OFFSET, 0.1),
        Transform::from_xyz(0., 0., 0.1),
    ]
    .into_iter();

    card.available_choices()
        .iter()
        .map(|&ch| {
            spawn_shape(
                card.shape(ch).unwrap(),
                &mut com,
                &assets,
                ch,
                transforms.next().unwrap(),
            )
        })
        .collect()
}

fn setup_ruin(com: &mut Commands, assets: &Res<AssetManager>) -> Vec<Entity> {
    // change the background of the selection area?
    Vec::default()
}

fn setup_normal(card: &Card, mut com: &mut Commands, assets: &Res<AssetManager>) -> Vec<Entity> {
    let left_transform = Transform::from_xyz(0., -150., 0.1);
    let right_transform = Transform::from_xyz(0., 150., 0.1);

    let left = card.shape(Choice::Left).unwrap();
    let right = card.shape(Choice::Right).unwrap();
    vec![
        spawn_shape(left, &mut com, &assets, Choice::Left, left_transform),
        spawn_shape(right, &mut com, &assets, Choice::Right, right_transform),
    ]
}

fn handle_cultivation(
    mut com: Commands,
    mut grid: ResMut<CGrid>,
    query: Query<Entity, With<Shape>>,
    mut network_events: EventReader<NetworkData<CultivationCommand>>,
    assets: Res<AssetManager>,
    mut handles: Query<&mut Handle<ColorMaterial>>,
    mut states: ResMut<State<ClientGameState>>,
) {
    for cultivation_event in network_events.iter() {

            if cultivation_event.player == CLIENT_NAME {
            info!("Cultivate self");
            grid.cultivate(
                (cultivation_event as &CultivationCommand).clone(),
                &assets,
                &mut handles,
            );
            // TODO Racecondition??: 
            // If cultivation comes one tick AFTER new card was sent, this deletes new cards as well
            // however, tcp/spicy guarantee, that both packages arrive at the same tick worst
            com.remove_resource::<ActiveShape>();

            for e in query.iter() {
                com.entity(e).despawn_recursive();
            }
        } else {
            info!("Cultivate other");
        }
    }
}

// This is because of a bug in `SystemState::on_in_stack_update` which runs not when it should, so we reimplement it :)
fn play_state(state:Res<State<ClientGameState>>)-> ShouldRun{
    if state.current() == &ClientGameState::Playing || state.inactives().contains(&ClientGameState::Playing){
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn handle_card(
    mut com: Commands,
    mut network_events: EventReader<NetworkData<Card>>,
    mut state: ResMut<State<ClientGameState>>,
) {
    for card_event in network_events.iter() {
        let card: Card = (card_event as &Card).clone();
        info!("Received Card");
        state.push(ClientGameState::ActiveTurn).unwrap();
        com.insert_resource(card);
    }
}
