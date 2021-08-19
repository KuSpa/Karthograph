use bevy::prelude::*;

use crate::{asset_management::AssetManager, objective::GameObjectives, seasons::SeasonType};

pub fn setup_ui(mut com: Commands) {
    com.spawn_bundle(UiCameraBundle::default());
}

pub fn setup_objective_ui(
    mut com: Commands,
    assets: Res<AssetManager>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let objectives = GameObjectives::default();
    // TODO BRING THIS SOMEWHERE ELSE
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
        setup_season_ui(
            &mut parent,
            SeasonType::Winter,
            &assets,
            &mut materials,
            &objectives,
        );
        setup_season_ui(
            &mut parent,
            SeasonType::Autumn,
            &assets,
            &mut materials,
            &objectives,
        );
        setup_season_ui(
            &mut parent,
            SeasonType::Summer,
            &assets,
            &mut materials,
            &objectives,
        );
        setup_season_ui(
            &mut parent,
            SeasonType::Spring,
            &assets,
            &mut materials,
            &objectives,
        );
    });
    com.insert_resource(objectives);
}

fn setup_season_ui(
    child_builder: &mut ChildBuilder,
    season: SeasonType,
    assets: &Res<AssetManager>,
    materials: &mut Assets<ColorMaterial>,
    objectives: &GameObjectives,
) {
    // one line as name and then two lines for both objectives
    let text_style = TextStyle {
        font: assets.font.clone(),
        font_size: 60.0,
        color: Color::BLACK,
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
            material: materials.add(Color::rgba(0.0, 0.8, 0.65, 0.5).into()),

            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(second_objective).insert(marker);
            parent.spawn_bundle(first_objective).insert(marker);
            parent.spawn_bundle(season_name);
        });
}
