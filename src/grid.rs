use crate::asset_management::AssetManager;
use crate::card::RuinIndicator;
use crate::shape::{Geometry, Shape};
use crate::util::to_array;
use bevy::math::i32;
use bevy::prelude::*;
use derive_deref::*;
use serde::Deserialize;
use std::collections::VecDeque;
use std::ops::{Add, RangeFrom};
use std::usize;

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: usize = 11;
//x=y offset
const GRID_OFFSET: f32 = SPRITE_SIZE;
#[derive(Debug, Default, Clone, Copy, Deserialize, Deref, DerefMut)]
pub struct Coordinate(IVec2);

impl Coordinate {
    pub fn inner_copy(&self) -> IVec2 {
        self.0
    }
}

impl From<(usize, usize)> for Coordinate {
    fn from((x, y): (usize, usize)) -> Self {
        Self(IVec2::new(x as i32, y as i32))
    }
}

impl From<(i32, i32)> for Coordinate {
    fn from((x, y): (i32, i32)) -> Self {
        Self(IVec2::new(x, y))
    }
}

impl From<IVec2> for Coordinate {
    fn from(val: IVec2) -> Self {
        Self(val)
    }
}

impl From<Vec2> for Coordinate {
    fn from(val: Vec2) -> Self {
        Self(val.as_i32())
    }
}

impl Add<Coordinate> for Coordinate {
    type Output = Coordinate;
    fn add(self, rhs: Coordinate) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum Cultivation {
    //Add a None type??
    Village,
    Water,
    Farm,
    Forest,
    Goblin,
}

impl From<Cultivation> for &'static str {
    fn from(cult: Cultivation) -> &'static str {
        match cult {
            Cultivation::Village => "village",
            Cultivation::Water => "water",
            Cultivation::Forest => "forest",
            Cultivation::Farm => "farm",
            Cultivation::Goblin => "goblin",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Terrain {
    Normal,
    Mountain,
    Ruin,
}
impl Default for Terrain {
    fn default() -> Self {
        Terrain::Normal
    }
}

impl From<Terrain> for &'static str {
    fn from(terrain: Terrain) -> &'static str {
        match terrain {
            Terrain::Mountain => "mountain",
            Terrain::Normal => "default",
            Terrain::Ruin => "ruin",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CultivationInformation {
    cultivation: Cultivation,
    area_id: usize,
    size: usize,
}

impl CultivationInformation {
    pub fn cultivation(&self) -> &Cultivation {
        &self.cultivation
    }
}

impl From<Cultivation> for CultivationInformation {
    fn from(c: Cultivation) -> Self {
        Self {
            cultivation: c,
            area_id: 0,
            size: 0,
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub cultivation: Option<CultivationInformation>,
    terrain: Terrain,
    entity: Entity,
    position: Coordinate,
}
struct FieldComponent;

impl Field {
    fn new(entity: Entity, position: Coordinate) -> Self {
        Field {
            entity,
            terrain: Terrain::default(),
            cultivation: Option::default(),
            position,
        }
    }

    pub fn position(&self) -> Coordinate {
        self.position
    }
}

#[derive(Debug)]
pub struct Grid {
    entity: Entity,
    area_counter: RangeFrom<usize>,
    inner: [Field; Grid::SIZE * Grid::SIZE],
}

impl Grid {
    const SIZE: usize = GRID_SIZE as usize;

    pub fn screen_to_grid(mut position: Vec2) -> Coordinate {
        position.x -= GRID_OFFSET;
        position.y -= GRID_OFFSET;
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

    fn is_valid_coord(&self, coord: &Coordinate) -> bool {
        !(coord.x < 0
            || coord.x >= Self::SIZE as i32
            || coord.y < 0
            || coord.y >= Self::SIZE as i32)
    }

    fn index(&self, coord: &Coordinate) -> Result<usize, ()> {
        if !self.is_valid_coord(&coord) {
            Err(())
        } else {
            Ok((coord.x as usize) + (coord.y as usize) * Self::SIZE)
        }
    }

    pub fn is_ruin(&self, coord: &Coordinate) -> bool {
        if let Ok(index) = self.index(coord) {
            self.inner[index].terrain == Terrain::Ruin
        } else {
            false
        }
    }

    pub fn is_free(&self, coord: &Coordinate) -> bool {
        if let Ok(index) = self.index(coord) {
            let field = &self.inner[index];
            field.terrain != Terrain::Mountain && field.cultivation.is_none()
        } else {
            false
        }
    }

    pub fn try_cultivate(
        &mut self,
        shape: &Shape,
        coord: &Coordinate,
        assets: &AssetManager,
        mut handles: &mut Query<&mut Handle<ColorMaterial>>,
    ) -> Result<(), &'static str> {
        if self.accepts_geometry_at(&shape.geometry(), &coord, &shape.ruin()) {
            self.cultivate(&shape, &coord, &assets, &mut handles);
            Ok(())
        } else {
            Err("Can't place the shape here")
        }
    }

    fn cultivate(
        &mut self,
        shape: &Shape,
        coord: &Coordinate,
        assets: &AssetManager,
        handles: &mut Query<&mut Handle<ColorMaterial>>,
    ) {
        for position in shape.geometry().iter() {
            let mut field = self.at_mut(&(*coord + *position)).unwrap();
            field.cultivation = Some(shape.cultivation().into());
            let mut handle = handles.get_mut(field.entity).unwrap();
            *handle = assets.fetch(shape.cultivation().into()).unwrap();
        }

        let id = self.area_counter.next().unwrap();
        self.propagate_id(&coord, id, &shape.cultivation());
    }

    fn propagate_id(&mut self, coord: &Coordinate, id: usize, cultivation: &Cultivation) {
        let mut queue = VecDeque::new();
        let mut fields = Vec::new();
        queue.push_front(*coord);
        while let Some(pos) = queue.pop_back() {
            fields.push(pos);

            //set current to

            // non valid `pos` are not added to the queue
            self.at_mut(&pos)
                .as_mut()
                .unwrap()
                .cultivation
                .as_mut()
                .unwrap()
                .area_id = id;

            for neighbor_pos in self.neighbor_indices(&pos) {
                let field = self.at(&neighbor_pos).unwrap();

                if let Some(area_info) = field.cultivation {
                    if area_info.area_id < id && area_info.cultivation == *cultivation {
                        queue.push_front(neighbor_pos);
                    }
                }
            }
        }

        let size = fields.len();
        for field_pos in fields {
            self.at_mut(&field_pos)
                .as_mut()
                .unwrap()
                .cultivation
                .as_mut()
                .unwrap()
                .size = size;
        }
    }

    // I would like to have an iterator to `&Field` here, but then I would have to `split` which is unpleasant, so this has to suffice :(
    fn neighbor_indices(&self, coord: &Coordinate) -> Vec<Coordinate> {
        let top = *coord + (0, 1).into();
        let bottom = *coord + (0, -1).into();
        let right = *coord + (1, 0).into();
        let left = *coord + (-1, 0).into();
        vec![top, bottom, right, left]
            .into_iter()
            .filter(|c| self.is_valid_coord(&c))
            .collect()
    }

    pub fn accepts_geometry(&self, geom: &Geometry, ruins: &RuinIndicator) -> bool {
        // little bit ugly as the grid assumes knowledge on what orientations the geometry can have, but hm
        let mut mirrored = geom.clone();
        mirrored.mirror();
        for mut geom in [geom.clone(), mirrored] {
            for _ in 0..4 {
                for field in self.all() {
                    if self.accepts_geometry_at(&geom, &field.position, &ruins) {
                        return true;
                    }
                }
                geom.rotate_clockwise();
            }
        }
        false
    }

    pub fn accepts_geometry_at(
        &self,
        geom: &Geometry,
        coord: &Coordinate,
        ruins: &RuinIndicator,
    ) -> bool {
        let mut on_ruin = false;
        for &pos in geom.iter() {
            if self.is_ruin(&(pos + *coord)) {
                on_ruin = true;
            }
            if !self.is_free(&(pos + *coord)) {
                return false;
            }
        }
        // == !(ruin && !on_ruin) <= if it SHOULD be on a ruin, but is NOT, then return false
        !(**ruins) || on_ruin
    }

    fn initialize(com: &mut Commands, ruins: &[Coordinate], mountains: &[Coordinate]) -> Self {
        let mut temp_vec: Vec<Field> = Vec::default();
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                let entity = com.spawn().id();
                temp_vec.push(Field::new(entity, (x, y).into()));
            }
        }
        let entity = com.spawn().id();
        let mut grid = Grid {
            entity,
            area_counter: 1..,
            inner: to_array::<Field, { Self::SIZE * Self::SIZE }>(temp_vec),
        };

        for pos in mountains.iter() {
            grid.inner[grid.index(&pos).unwrap()].terrain = Terrain::Mountain;
        }

        for pos in ruins.iter() {
            grid.inner[grid.index(&pos).unwrap()].terrain = Terrain::Ruin;
        }
        grid
    }

    pub fn at_mut(&mut self, coord: &Coordinate) -> Result<&mut Field, ()> {
        self.index(&coord).map(move |i| &mut self.inner[i])
    }

    pub fn at(&self, coord: &Coordinate) -> Result<&Field, ()> {
        self.index(&coord).map(|i| &self.inner[i])
    }

    pub fn new(com: &mut Commands) -> Self {
        let mountains: Vec<Coordinate> = vec![
            (2, 2).into(),
            (3, 9).into(),
            (5, 5).into(),
            (7, 1).into(),
            (8, 8).into(),
        ]; //(x,y)
        let ruins: Vec<Coordinate> = vec![
            (1, 2).into(),
            (1, 8).into(),
            (5, 1).into(),
            (5, 9).into(),
            (9, 2).into(),
            (9, 8).into(),
        ];
        Grid::initialize(com, &ruins, &mountains)
    }

    pub fn all(&self) -> impl Iterator<Item = &Field> {
        self.inner.iter()
    }

    // use result for safety?
    pub fn row(&self, nth: usize) -> impl Iterator<Item = &Field> {
        self.inner.iter().skip(nth * Self::SIZE).take(Self::SIZE)
    }

    // use result for safety?
    pub fn column(&self, nth: usize) -> impl Iterator<Item = &Field> {
        self.inner.iter().skip(nth).step_by(Self::SIZE)
    }
}

pub fn init_grid(mut com: Commands, assets: Res<AssetManager>) {
    let grid = Grid::new(&mut com);

    for x in 0..(GRID_SIZE as usize) {
        for y in 0..(GRID_SIZE as usize) {
            let field = grid.at(&(x, y).into()).unwrap();
            let mat = assets.fetch(field.terrain.into()).unwrap();
            let entity = field.entity;
            //THE FIELD OR THE GRID SHOULD DO THIS ITSELF
            com.entity(entity)
                .insert(FieldComponent)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    material: mat,
                    transform: Transform::from_xyz(
                        x as f32 * SPRITE_SIZE + GRID_OFFSET,
                        y as f32 * SPRITE_SIZE + GRID_OFFSET,
                        -0.1,
                    ),
                    ..Default::default()
                });
        }
    }
    com.insert_resource(grid);
}
