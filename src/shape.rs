use derive_deref::*;
use serde::Deserialize;

use crate::asset_management::AssetManager;
use crate::card::{Card, RuinIndicator};
use crate::grid::{Coordinate, Cultivation, FieldComponent, Grid};
use crate::util::min_f;
use crate::SPRITE_SIZE;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
//TODO MIRROR

#[derive(Clone, Deserialize, Deref, DerefMut)]
pub struct Geometry {
    // TODO should this know the size its drawn??
    pub inner: Vec<Coordinate>,
}
impl Geometry {
    pub fn rotate_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp().perp().perp().into();
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp().into();
        }
    }

    pub fn mirror(&mut self) {
        for position in self.iter_mut() {
            position.x = -position.x;
        }
    }

    pub fn as_transforms_centered(&self, distance: f32, z: f32) -> Vec<Transform> {
        let offset = self.center_offset();
        let mut transforms = self.as_transforms(distance, z);
        transforms
            .iter_mut()
            .for_each(|trans| trans.translation -= offset * distance);

        transforms
    }

    pub fn as_transforms(&self, distance: f32, z: f32) -> Vec<Transform> {
        self.iter()
            .map(|pos| Transform::from_xyz(distance * pos.x as f32, distance * pos.y as f32, z))
            .collect()
    }

    fn min_max(&self) -> (IVec2, IVec2) {
        let mut max_v = IVec2::ZERO;
        let mut min_v = IVec2::ZERO;
        for coord in self.iter() {
            min_v = min_v.min(coord.inner_copy());
            max_v = max_v.max(coord.inner_copy());
        }
        (min_v, max_v)
    }

    pub fn max_size_in_rect(&self, size: Vec2) -> f32 {
        let (min_v, max_v) = self.min_max();
        let diff = max_v - min_v + IVec2::ONE;
        //calculate the largest possible square size
        min_f(size.x / (diff.x as f32), size.y / (diff.y as f32))
    }

    fn center_offset(&self) -> Vec3 {
        let (min_v, max_v) = self.min_max();
        let offset = max_v.as_f32() - (max_v.as_f32() - min_v.as_f32()) / 2.;
        Vec3::new(offset.x as f32, offset.y as f32, 0.)
    }
}

#[derive(Clone)]
pub struct Shape {
    pub geometry: Geometry,
    pub cultivation: Cultivation,
    pub ruin: RuinIndicator,
}

impl Shape {
    pub fn new(g: &Geometry, cult: &Cultivation, ruin: &RuinIndicator) -> Self {
        Self {
            geometry: g.clone(),
            cultivation: *cult,
            ruin: *ruin,
        }
    }

    pub fn spawn(self, com: &mut Commands, assets: &Res<AssetManager>) -> Entity {
        let handle = assets.fetch(self.cultivation.into()).unwrap();

        let mut children = Vec::<Entity>::new();
        let mat = handle;

        for &transform in self.geometry.as_transforms(SPRITE_SIZE, 0.).iter() {
            let child = com
                .spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    material: mat.clone(),
                    transform,
                    ..Default::default()
                })
                .id();
            children.push(child);
        }
        // TODO: spawn with transform??????
        // FIXME: if no new mouse event triggers, transform is not set...
        com.spawn()
            .insert(self)
            .insert(GlobalTransform::default())
            .insert(Transform::default())
            .push_children(&children)
            .id()
    }

    // POSSIBLE BREAK -> CALLER DETERMINES WHAT TRANSFORMS TO ROTATE
    pub fn rotate_clockwise(&mut self, transforms: &mut [Mut<Transform>]) {
        for transform in transforms.iter_mut() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            transform.translation.x = y;
            transform.translation.y = -x;
        }
        self.geometry.rotate_clockwise();
    }

    // POSSIBLE BREAK -> CALLER DETERMINES WHAT TRANSFORMS TO ROTATE
    pub fn rotate_counter_clockwise(&mut self, transforms: &mut [Mut<Transform>]) {
        for transform in transforms.iter_mut() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            transform.translation.x = -y;
            transform.translation.y = x;
        }
        self.geometry.rotate_counter_clockwise();
    }

    pub fn mirror(&mut self, transforms: &mut [Mut<Transform>]) {
        for transform in transforms.iter_mut() {
            transform.translation.x = -transform.translation.x
        }
        self.geometry.mirror();
    }
}

impl Default for Shape {
    fn default() -> Self {
        let geometry = Geometry {
            inner: vec![(1, 0).into(), (0, 1).into(), (0, 0).into()],
        };
        Self {
            geometry,
            cultivation: Cultivation::Village,
            ruin: false.into(),
        }
    }
}

pub fn move_shape(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<(&Shape, &mut Transform)>,
    grid: Res<Grid>,
) {
    // BREAKS IF TWO SHAPES ARE ACTIVE
    if let Ok((shape, mut transform)) = query.single_mut() {
        for event in cursor.iter() {
            //calculate the closest cell
            let mut position = event.position;
            let grid_pos = Grid::screen_to_grid(position);
            if grid.accepts_geometry_at(&shape.geometry, &grid_pos, &shape.ruin) {
                position = Grid::grid_to_screen(grid_pos);
            }
            // IF CANNOT PLACE => DONT MOVE

            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }
    }
}

pub fn rotate_shape(
    mut cursor: EventReader<MouseWheel>,
    mut parents: Query<(Entity, &mut Shape)>,
    mut query: Query<(&Parent, &mut Transform)>,
) {
    if let Ok((parent, mut shape)) = parents.single_mut() {
        let mut transforms: Vec<Mut<Transform>> = query
            .iter_mut()
            .filter_map(|(Parent(ent), tr)| if *ent == parent { Some(tr) } else { None })
            .collect();

        for event in cursor.iter() {
            //calculate the closest cell
            if event.y < 0. {
                // CW
                shape.rotate_clockwise(&mut transforms);
            } else {
                shape.rotate_counter_clockwise(&mut transforms);
            }
        }
    }
}

pub fn mirror_shape(
    mut clicks: EventReader<MouseButtonInput>,
    mut parents: Query<(Entity, &mut Shape)>,
    mut query: Query<(&Parent, &mut Transform)>,
) {
    if let Ok((parent, mut shape)) = parents.single_mut() {
        let mut transforms: Vec<Mut<Transform>> = query
            .iter_mut()
            .filter_map(|(Parent(ent), tr)| if *ent == parent { Some(tr) } else { None })
            .collect();

        for event in clicks.iter() {
            //calculate the closest cell
            if event.button == MouseButton::Middle && event.state.is_pressed() {
                shape.mirror(&mut transforms);
            }
        }
    }
}

pub fn place_shape(
    mut com: Commands,
    shapes: Query<(Entity, &Shape, &Transform)>,
    mut grid: ResMut<Grid>,
    card: Query<(Entity, &Card)>,
    mut clicks: EventReader<MouseButtonInput>,
    assets: Res<AssetManager>,
    mut fields: Query<(&FieldComponent, &mut Handle<ColorMaterial>)>,
) {
    for event in clicks.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            if let Ok((t_entity, shape, transform)) = shapes.single() {
                let position = Vec2::new(transform.translation.x, transform.translation.y);
                let grid_position = Grid::screen_to_grid(position);
                if let Ok(entities) = grid.try_cultivate(&shape, &grid_position) {
                    // Well we got through all the ifs and if lets, it's time to DO SOME STUFF
                    for &entity in entities.iter() {
                        let (_, mut handle) = fields.get_mut(entity).unwrap();
                        *handle = assets.fetch(shape.cultivation.into()).unwrap();
                    }
                    com.entity(t_entity).despawn_recursive();
                    if let Ok((card_entity, _)) = card.single() {
                        com.entity(card_entity).despawn_recursive();
                    }
                }
            }
        }
    }
}
