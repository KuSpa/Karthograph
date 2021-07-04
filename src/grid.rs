use crate::asset_management::AssetManager;
use crate::util::to_array;
use bevy::prelude::*;
use serde::Deserialize;
use std::usize;

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: i32 = 11;
//x=y offset
const GRID_OFFSET: f32 = SPRITE_SIZE;
pub type Coordinate = IVec2;

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
    pub inner: [[Field; GRID_SIZE as usize]; GRID_SIZE as usize],
}

impl Grid {
    pub fn screen_to_grid(mut position: Vec2) -> Coordinate {
        position.x -= GRID_OFFSET;
        position.y -= GRID_OFFSET;
        position /= SPRITE_SIZE;
        position = position.round();
        IVec2::new(position.x as i32, position.y as i32)
    }

    pub fn is_ruin(&self, coord: &Coordinate) -> bool {
        if !self.inbound(&coord) {
            return false;
        }
        self.inner[coord.x as usize][coord.y as usize].terrain == Terrain::Ruin
    }

    pub fn grid_to_screen(coord: Coordinate) -> Vec2 {
        let mut position = Vec2::new(GRID_OFFSET, GRID_OFFSET);
        position.x += coord.x as f32 * SPRITE_SIZE;
        position.y += coord.y as f32 * SPRITE_SIZE;
        position
    }

    fn inbound(&self, coord: &Coordinate) -> bool {
        !(coord.x >= GRID_SIZE || coord.y >= GRID_SIZE || coord.x < 0 || coord.y < 0)
    }

    pub fn is_free(&self, coord: &Coordinate) -> bool {
        if !self.inbound(coord) {
            return false;
        }
        let field = &self.inner[coord.x as usize][coord.y as usize];
        field.terrain != Terrain::Mountain && field.cultivation.is_none()
    }

    pub fn cultivate(&mut self, coord: &Coordinate, cultivation: &Cultivation) -> Entity {
        self.inner[coord.x as usize][coord.y as usize].cultivation = Some(*cultivation);
        self.inner[coord.x as usize][coord.y as usize].entity
    }

    fn initialize(com: &mut Commands, ruins: &[Coordinate], mountains: &[Coordinate]) -> Self {
        const SIZE: usize = GRID_SIZE as usize;
        let mut temp_vec: Vec<[Field; SIZE]> = Vec::default();
        for x in 0..GRID_SIZE {
            let mut col_vec: Vec<Field> = Vec::default();

            for y in 0..GRID_SIZE {
                let entity = com.spawn().id();
                col_vec.push(Field::new(entity, Coordinate::new(x as i32, y as i32)));
            }
            temp_vec.push(to_array::<Field, SIZE>(col_vec));
        }
        let entity = com.spawn().id();
        let mut grid = Grid {
            entity,
            inner: to_array::<[Field; SIZE], SIZE>(temp_vec),
        };

        for &pos in mountains.iter() {
            grid.inner[pos.x as usize][pos.y as usize].terrain = Terrain::Mountain;
        }

        for pos in ruins.iter() {
            grid.inner[pos.x as usize][pos.y as usize].terrain = Terrain::Ruin;
        }
        grid
    }

    pub fn new(com: &mut Commands) -> Self {
        let mountains: Vec<Coordinate> = vec![
            IVec2::new(2, 2),
            IVec2::new(3, 9),
            IVec2::new(5, 5),
            IVec2::new(7, 1),
            IVec2::new(8, 8),
        ]; //(x,y)
        let ruins: Vec<Coordinate> = vec![
            IVec2::new(1, 2),
            IVec2::new(1, 8),
            IVec2::new(5, 1),
            IVec2::new(5, 9),
            IVec2::new(9, 2),
            IVec2::new(9, 8),
        ];
        Grid::initialize(com, &ruins, &mountains)
    }

    /*How would I implement this using IntoIter???*/
    // TODO: make this a proper iter!
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.inner.iter().flat_map(|arr| arr.iter())
    }
}

pub fn init_grid(mut com: Commands, assets: Res<AssetManager>) {
    let grid = Grid::new(&mut com);

    for x in 0..(GRID_SIZE as usize) {
        for y in 0..(GRID_SIZE as usize) {
            let mat = assets.fetch(grid.inner[x][y].terrain.into()).unwrap();
            let entity = grid.inner[x][y].entity;
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
