use serde::Deserialize;
use std::ops::{Deref, DerefMut};

use crate::asset_management::AssetManager;
use crate::card::Card;
use crate::grid::{Coordinate, Cultivation, FieldComponent, Grid};
use crate::util::min_f;
use crate::SPRITE_SIZE;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
//TODO MIRROR

#[derive(Clone, Deserialize)]
pub struct Geometry {
    // TODO should this know the size its drawn??
    pub inner: Vec<Coordinate>,
}
impl Geometry {
    pub fn rotate_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp().perp().perp();
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        for position in self.iter_mut() {
            *position = position.perp();
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
            min_v = min_v.min(*coord);
            max_v = max_v.max(*coord);
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

impl Deref for Geometry {
    type Target = Vec<Coordinate>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Geometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
#[derive(Clone)]
pub struct Shape {
    pub geometry: Geometry,
    pub cultivation: Cultivation,
    pub ruin: bool,
}

impl Shape {
    pub fn new(g: &Geometry, cult: &Cultivation, ruin: bool) -> Self {
        Self {
            geometry: g.clone(),
            cultivation: *cult,
            ruin,
        }
    }

    pub fn spawn(self, com: &mut Commands, assets: &Res<AssetManager>) -> Entity {
        //TODO PROPER TEXTURE
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

    fn is_placable(&self, grid: &Grid, coord: &Coordinate, ruin: bool) -> bool {
        let mut on_ruin = false;
        for &pos in self.geometry.iter() {
            if grid.is_ruin(&(pos + *coord)) {
                on_ruin = true;
            }
            if !grid.is_free(&(pos + *coord)) {
                return false;
            }
        }
        // == !(ruin && !on_ruin) <= if it SHOULD be on a ruin, but is NOT, then return false
        !ruin || on_ruin
    }

    fn try_place(
        &self,
        grid: &mut Grid,
        position: &Coordinate,
        ruin: bool,
    ) -> Result<Vec<Entity>, &'static str> {
        //return an error(?)
        if !self.is_placable(&grid, &position, ruin) {
            return Err("Can't place the shape here");
        }
        Ok(self
            .geometry
            .iter()
            .map(|&pos| grid.cultivate(&(pos + *position), &self.cultivation))
            .collect())
    }
}

impl Default for Shape {
    fn default() -> Self {
        let geometry = Geometry {
            inner: vec![IVec2::new(1, 0), IVec2::new(0, 1), IVec2::new(0, 0)],
        };
        Self {
            geometry,
            cultivation: Cultivation::Village,
            ruin: false,
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
            if shape.is_placable(&grid, &grid_pos, shape.ruin) {
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
                if let Ok(entities) = shape.try_place(&mut grid, &grid_position, shape.ruin) {
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
