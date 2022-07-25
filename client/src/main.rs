mod asset_management;
mod network;
mod phases;
mod shape;
mod ui;

use std::convert::TryFrom;

use bevy::{math::Vec2, window::CursorMoved};

use asset_management::{AssetID, AssetManager, AssetPlugin};
use bevy::{prelude::*, render::camera::WindowOrigin};
use derive_deref::*;
use common::grid::{Coordinate, Field, GridLike, GRID_SIZE};
use common::network::CultivationCommand;
use common::util::to_array;
use network::ClientNetworkPlugin;
use phases::ActivePhasePlugin;
use ui::CustomUIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ClientNetworkPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(ActivePhasePlugin)
        .add_plugin(CustomUIPlugin)
        .add_startup_system(init_camera.system())
        .insert_resource(MousePosition::default())
        .add_system(mouse_position.system())
        .run();
    /*
    .add_event::<NewCard>()
    .insert_resource(GameObjectives::default())
    .insert_resource(Season::default())
    .insert_resource(RuinIndicator::default())


    .add_startup_system(setup_ui.system())
    .add_system_set(SystemSet::on_exit(GameState::Loading).with_system(init_grid.system()))
    .add_system_set(
        SystemSet::on_enter(GameState::SeasonState)
            .with_system(initialize_cards.system())
            .with_system(setup_objective_ui.system()),
    )
    .add_system_set(
        SystemSet::on_resume(GameState::SeasonState).with_system(initialize_cards.system()),
    )
    .add_system_set(
        SystemSet::on_update(GameState::SeasonState)
            .with_system(next_card.system())
            .with_system(move_shape.system())
            .with_system(mirror_shape.system())
            .with_system(rotate_shape.system())
            .with_system(place_shape.system())
            .with_system(mouse_position.system())
            .with_system(click_card.system()),
    )
    .add_system_set(
        SystemSet::on_enter(GameState::SeasonScommonState).with_system(score_season.system()),
    )
    .add_system_set(
        SystemSet::on_exit(GameState::SeasonScoreState).with_system(advance_season.system()),
    )
    /*.add_system(
        spawn_shape
            .system()
            .config(|params| params.2 = Some(Timer::new(Duration::from_secs_f32(0.1), false))),
    )*/
    .run();*/
}

pub(crate) const SPRITE_SIZE: f32 = 75.;
pub(crate) const GRID_OFFSET: f32 = 75.;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ClientGameState {
    Loading,
    Playing,
    ActiveTurn,
    Waiting,
    Scoring,
}

impl Default for ClientGameState{
    fn default() -> Self {
        Self::Loading
    }
}

fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}

pub(crate) struct CField {
    pub field: Field,
    pub entity: Entity,
}

struct FieldComponent;

pub(crate) struct CGrid {
    entity: Entity,
    inner: [CField; GRID_SIZE * GRID_SIZE],
}

impl CGrid {
    fn new(
        original: Vec<Field>,
        com: &mut Commands,
        assets: &AssetManager,
    ) -> Result<Self, &'static str> {
        if original.len() != GRID_SIZE * GRID_SIZE {
            return Err("Cannot convert Grid, Wrong size");
        }
        let arr = to_array::<CField, { GRID_SIZE * GRID_SIZE }>(
            original
                .into_iter()
                .map(|field| {
                    let material = assets.fetch(field.terrain.asset_id()).unwrap();
                    let p = field.position;
                    let entity = com
                        .spawn()
                        .insert(FieldComponent)
                        .insert_bundle(SpriteBundle {
                            sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                            material,
                            transform: Transform::from_xyz(
                                p.x as f32 * SPRITE_SIZE,
                                p.y as f32 * SPRITE_SIZE,
                                -0.1,
                            ),
                            ..Default::default()
                        })
                        .id();
                    CField { entity, field }
                })
                .collect::<Vec<_>>(),
        );
        let children = arr.iter().map(|f| f.entity).collect::<Vec<_>>();

        let e = com
            .spawn()
            .insert(Transform::from_xyz(GRID_OFFSET, GRID_OFFSET, 0.))
            .insert(GlobalTransform::default())
            .push_children(&children)
            .id();

        Ok(CGrid {
            entity: e,
            inner: arr,
        })
    }

    pub fn cultivate(
        &mut self,
        command: CultivationCommand,
        assets: &AssetManager,
        handles: &mut Query<&mut Handle<ColorMaterial>>,
    ) {
        for position in command.geometry.iter() {
            let ref mut field = self.inner[self.index(&(*position + command.position)).unwrap()];
            field.field.cultivation = Some(command.cultivation);
            let mut handle = handles.get_mut(field.entity).unwrap();
            *handle = assets.fetch(command.cultivation.asset_id()).unwrap();
        }
    }

    pub fn screen_to_grid(mut position: Vec2) -> Coordinate {
        position.x -= GRID_OFFSET;
        position.y -= GRID_OFFSET;
        //position;// -= SPRITE_SIZE/Vec2::new(2.,2.);
        position /= SPRITE_SIZE;
        position = position.round();
        position.into()
    }

    pub fn grid_to_screen(coord: Coordinate) -> Vec2 {
        let mut position = Vec2::new(GRID_OFFSET, GRID_OFFSET);
        position.x += coord.x as f32 * SPRITE_SIZE;
        position.y += coord.y as f32 * SPRITE_SIZE;
        position
    }
}

impl GridLike for CGrid {
    fn is_free(&self, c: &Coordinate) -> bool {
        if let Ok(index) = self.index(c) {
            self.inner[index].field.is_free()
        } else {
            false
        }
    }

    fn is_ruin(&self, c: &Coordinate) -> bool {
        if let Ok(index) = self.index(c) {
            self.inner[index].field.is_ruin()
        } else {
            false
        }
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct MousePosition(Vec2);

pub fn mouse_position(mut mouse: ResMut<MousePosition>, mut cursor: EventReader<CursorMoved>) {
    if let Some(event) = cursor.iter().last() {
        *mouse = MousePosition(event.position);
    };
}
