use std::collections::HashMap;
use std::convert::TryInto;
use std::usize;

use bevy::asset::AssetPath;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: usize = 11;

type Coordinate = (usize, usize);

//https://stackoverflow.com/a/29570662/5862030
fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(init_assets.system())
        .add_startup_system(init_camera.system())
        .add_startup_system(init_grid.system())
        .add_system(move_tiles.system())
        .run();
}
#[derive(Debug)]
enum Cultivation {
    //Add a None type??
    Village,
    River,
    Farm,
    Goblins,
}
#[derive(Debug, Copy, Clone)]
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
impl Into<&'static str> for Terrain {
    fn into(self) -> &'static str {
        match self {
            Terrain::Mountain => "mountain",
            Terrain::Normal => "default",
            Terrain::Ruin => "ruin",
        }
    }
}

enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
struct Field {
    cultivation: Option<Cultivation>,
    terrain: Terrain,
    entity: Entity,
    position: Coordinate,
}

impl Field {
    fn new(entity: Entity, x: usize, y: usize) -> Self {
        Field {
            entity,
            terrain: Terrain::default(),
            cultivation: Option::default(),
            position: (x, y),
        }
    }
}

#[derive(Debug)]
struct Grid {
    pub entity: Entity,
    pub inner: [[Field; GRID_SIZE]; GRID_SIZE],
}

struct Tile {
    geometry: Vec<Coordinate>,
    cultivation: Cultivation,
    orientation: Orientation,
}

impl Tile {
    fn spawn(com: &mut Commands, tile: Self, handle: Handle<ColorMaterial>) -> Entity {
        //TODO ASSETMANAGEMENT
        let mut children = Vec::<Entity>::new();
        let mat = handle;

        for &(x, y) in tile.geometry.iter() {
            let child = com
                .spawn()
                .insert_bundle(SpriteBundle {
                    material: mat.clone(),
                    transform: Transform::from_xyz(
                        x as f32 * SPRITE_SIZE,
                        y as f32 * SPRITE_SIZE,
                        0.,
                    ),
                    ..Default::default()
                })
                .id();
            children.push(child);
        }
        com.spawn()
            .insert(tile)
            .insert(GlobalTransform::default())
            .insert(Transform::default())
            .push_children(&children)
            .id()
    }
}

impl Default for Tile {
    fn default() -> Self {
        let geometry = vec![(0, 0), (0, 1)];
        Self {
            geometry,
            cultivation: Cultivation::Farm,
            orientation: Orientation::Up,
        }
    }
}

impl Grid {
    fn initialize(
        com: &mut Commands,
        ruins: &Vec<Coordinate>,
        mountains: &Vec<Coordinate>,
    ) -> Self {
        let mut temp_vec: Vec<[Field; GRID_SIZE]> = Vec::default();
        for x in 0..GRID_SIZE {
            let mut col_vec: Vec<Field> = Vec::default();

            for y in 0..GRID_SIZE {
                let entity = com.spawn().id();
                col_vec.push(Field::new(entity, x, y));
            }
            temp_vec.push(to_array::<Field, GRID_SIZE>(col_vec));
        }
        let entity = com.spawn().id();
        let mut grid = Grid {
            entity,
            inner: to_array::<[Field; GRID_SIZE], GRID_SIZE>(temp_vec),
        };

        for &(x, y) in mountains.iter() {
            grid.inner[x][y].terrain = Terrain::Mountain;
        }

        for &(x, y) in ruins.iter() {
            grid.inner[x][y].terrain = Terrain::Ruin;
        }
        grid
    }

    pub fn new(com: &mut Commands) -> Self {
        let mountains: Vec<Coordinate> = vec![(2, 2), (3, 9), (5, 5), (7, 1), (8, 8)]; //(x,y)
        let ruins: Vec<Coordinate> = vec![(1, 2), (1, 8), (5, 1), (5, 9), (9, 2), (9, 8)];
        Grid::initialize(com, &ruins, &mountains)
    }
}
fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}

const ASSETS: [(&'static str, &'static str); 4] = [
    ("mountain", "mountain.png"),
    ("ruin", "ruin.png"),
    ("default", "default.png"),
    ("farm", "farm.png"),
];

#[derive(Default)]
struct AssetManager {
    map: HashMap<&'static str, Handle<ColorMaterial>>,
}
impl AssetManager {
    fn insert_asset(&mut self, name: &'static str, handle: Handle<ColorMaterial>) {
        self.map.insert(name, handle);
    }
    fn fetch(&self, name: &'static str) -> Option<Handle<ColorMaterial>> {
        self.map.get(name).cloned()
    }

    fn new(asset_server: Res<AssetServer>, mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        let mut manager = Self::default();
        for (name, path) in ASSETS {
            let asset = materials.add(asset_server.load(path).clone().into());
            manager.insert_asset(name.into(), asset);
        }

        manager
    }
}

fn init_assets(mut com: Commands) {}

fn init_grid(
    mut com: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let grid = Grid::new(&mut com);

    let assets = AssetManager::new(asset_server, materials);

    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let mat = assets.fetch(grid.inner[x][y].terrain.into()).unwrap();
            let entity = grid.inner[x][y].entity;
            com.entity(entity).insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                material: mat,
                transform: Transform::from_xyz(
                    (x as f32 + 0.5) * SPRITE_SIZE,
                    (y as f32 + 0.5) * SPRITE_SIZE,
                    -0.1,
                ),
                ..Default::default()
            });
        }
    }

    Tile::spawn(&mut com, Tile::default(), assets.fetch("farm").unwrap());

    com.insert_resource(grid);
}

fn move_tiles(mut cursor: EventReader<CursorMoved>, mut query: Query<(&Tile, &mut Transform)>) {
    // we are sure, that there is only one tile active
    let mut transform = query.iter_mut().last().unwrap().1;

    for event in cursor.iter() {
        let position = event.position;
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}
