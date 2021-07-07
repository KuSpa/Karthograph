use crate::asset_management::AssetManager;
use crate::util::to_array;
use bevy::prelude::*;
use derive_deref::*;
use serde::Deserialize;
use std::ops::Add;
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

#[derive(Debug, Clone, Copy, Deserialize)]
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
pub enum Terrain {
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

#[derive(Debug)]
pub struct Field {
    pub cultivation: Option<Cultivation>,
    pub terrain: Terrain,
    pub entity: Entity, // TODO GETTER
    pub position: Coordinate,
}
pub struct FieldComponent;

impl Field {
    fn new(entity: Entity, position: Coordinate) -> Self {
        Field {
            entity,
            terrain: Terrain::default(),
            cultivation: Option::default(),
            position,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub entity: Entity,
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

    fn index(&self, coord: &Coordinate) -> Result<usize, ()> {
        if coord.x < 0
            || coord.x >= Self::SIZE as i32
            || coord.y < 0
            || coord.y >= Self::SIZE as i32
        {
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

    pub fn cultivate(&mut self, coord: &Coordinate, cultivation: &Cultivation) -> Entity {
        let index = self.index(coord).unwrap();
        self.inner[index].cultivation = Some(*cultivation);
        self.inner[index].entity
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
