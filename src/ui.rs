use bevy::prelude::*;

use crate::{asset_management::AssetManager, objective::GameObjectives, seasons::SeasonType};

pub fn setup_ui(mut com: Commands) {
    com.spawn_bundle(UiCameraBundle::default());
}

pub fn setup_objective_ui(
    mut com: Commands,
    objectives: Res<GameObjectives>, // If they are not yet initialized, Bevy will handle this for us
    assets: Res<AssetManager>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    com.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(25.0), Val::Percent(100.0)),
            position: Rect {
                left: Val::Percent(75.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        material: materials.add(Color::rgba(0.65, 0.65, 0.65, 0.5).into()),
        ..Default::default()
    })
    .with_children(|mut parent| {
        setup_season_ui(&mut parent, SeasonType::Winter, &assets, &objectives);
        setup_season_ui(&mut parent, SeasonType::Autumn, &assets, &objectives);
        setup_season_ui(&mut parent, SeasonType::Summer, &assets, &objectives);
        setup_season_ui(&mut parent, SeasonType::Spring, &assets, &objectives);
    });
}

pub struct SeasonUiMarker;

fn setup_season_ui(
    child_builder: &mut ChildBuilder,
    season: SeasonType,
    assets: &Res<AssetManager>,
    objectives: &GameObjectives,
) {
    // one line as name and then two lines for both objectives
    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
        color: Color::BLACK,
    };

    let color = if season == SeasonType::Spring {
        assets.ui.highlighted.clone()
    } else {
        assets.ui.default.clone()
    };

    // first objective
    let objective_name_a = objectives.objectives_for_season(&season).0.name();
    let objective_name_b = objectives.objectives_for_season(&season).1.name();

    let marker = season.marker();
    let season_sting = format!("{:?}", &season);
    let season_name = TextBundle {
        text: Text::with_section(season_sting, text_style.clone(), Default::default()),
        ..Default::default()
    };

    let first_objective = TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: objective_name_a.to_string(),
                    style: text_style.clone(),
                },
                TextSection {
                    value: "".to_string(),
                    style: text_style.clone(),
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let second_objective = TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: objective_name_b.to_string(),
                    style: text_style.clone(),
                },
                TextSection {
                    value: "".to_string(),
                    style: text_style.clone(),
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let coin_child = TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Coins".to_string(),
                    style: text_style.clone(),
                },
                TextSection {
                    value: "".to_string(),
                    style: text_style,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    // now for the actual compositing
    child_builder
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(98.0), Val::Percent(25.0)),
                margin: Rect::all(Val::Percent(1.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                align_content: AlignContent::SpaceBetween,
                ..Default::default()
            },
            material: color,

            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(coin_child).insert(marker);
            parent.spawn_bundle(second_objective).insert(marker);
            parent.spawn_bundle(first_objective).insert(marker);
            parent.spawn_bundle(season_name);
        })
        .insert(marker)
        .insert(SeasonUiMarker);
}
