use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;
use std::usize;

use bevy::core::Timer;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: usize = 11;
//x=y offset
const GRID_OFFSET: f32 = SPRITE_SIZE;

type Coordinate = (usize, usize);

//https://stackoverflow.com/a/29570662/5862030
fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system_to_stage(StartupStage::PreStartup, init_assets.system())
        .add_startup_system(init_camera.system())
        .add_startup_system(init_grid.system())
        .add_system(move_tiles.system())
        .add_system(place_tile.system())
        .add_system(spawn_tile.system().config(|params| params.2 =Some(Timer::new(Duration::from_secs_f32(0.5), false)) ))
        .run();
}

fn spawn_tile(
    mut com: Commands,
    query:Query<&Tile>,
    mut timer:Local<Timer>,
    time:ResMut<Time>,
    assets:Res<AssetManager>
){
    timer.tick(time.delta());

    if  query.iter().len() == 0 {
        if timer.just_finished() {
            let tile = Tile::default();
            Tile::spawn(&mut com, Tile::default(), assets.fetch(tile.cultivation.into()).unwrap());
        } else if timer.finished(){
            timer.reset();
        }
    }

}

fn place_tile(
    mut com: Commands,
    tiles: Query<(Entity, &Tile, &Transform)>,
    mut grid: ResMut<Grid>,
    mut clicks: EventReader<MouseButtonInput>,
    assets: Res<AssetManager>,
    mut fields: Query<(&FieldComponent, &mut Handle<ColorMaterial>)>,
) {
    for event in clicks.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            if let Ok((t_entity, tile, transform)) = tiles.single() {
                let position = Vec2::new(transform.translation.x, transform.translation.y);
                let grid_position = Grid::screen_to_grid(position);
                if let Ok(entities) = tile.try_place(&mut grid, &grid_position) {
                    // Well we got through all the ifs and if lets, it's time to DO SOME STUFF
                    for &entity in entities.iter() {
                        let (_, mut handle) = fields.get_mut(entity).unwrap();
                        *handle = assets.fetch(tile.cultivation.into()).unwrap();
                    }
                    com.entity(t_entity).despawn();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Cultivation {
    //Add a None type??
    Village,
    River,
    Farm,
    Goblin,
}

impl Into<&'static str> for Cultivation {
    fn into(self) -> &'static str {
        match self {
            Cultivation::Village => "village",
            Cultivation::River => "river",
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
//TODO MIRROR

#[derive(Debug)]
struct Field {
    cultivation: Option<Cultivation>,
    terrain: Terrain,
    entity: Entity,
    position: Coordinate,
}
struct FieldComponent;

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

impl Grid {
    fn screen_to_grid(mut position: Vec2) -> Coordinate {
        position.x -= GRID_OFFSET;
        position.y -= GRID_OFFSET;
        position /= SPRITE_SIZE;
        position = position.round();
        (position.x as usize, position.y as usize)
    }

    fn grid_to_screen(coord: Coordinate) -> Vec2 {
        let mut position = Vec2::new(GRID_OFFSET, GRID_OFFSET);
        position.x += coord.0 as f32 * SPRITE_SIZE;
        position.y += coord.1 as f32 * SPRITE_SIZE;
        position
    }

    fn is_free(&self, coord: &Coordinate) -> bool {
        let ref field = self.inner[coord.0][coord.1];
        field.terrain != Terrain::Mountain && field.cultivation.is_none()
    }

    fn cultivate(&mut self, coord: &Coordinate, cultivation: &Cultivation) -> Entity {
        self.inner[coord.0][coord.1].cultivation = Some(*cultivation);
        self.inner[coord.0][coord.1].entity.clone()
    }
}
struct Tile {
    geometry: Vec<Coordinate>,
    cultivation: Cultivation,
    orientation: Orientation,
}

impl Tile {
    // TODO make this non static
    fn spawn(com: &mut Commands, tile: Self, handle: Handle<ColorMaterial>) -> Entity {//TODO PROPER TEXTURE
        let mut children = Vec::<Entity>::new();
        let mat = handle;

        for &(x, y) in tile.geometry.iter() {
            let child = com
                .spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
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
        // TODO: spawn with transform?????? 
        // FIXME: if no new mouse event triggers, transform is not set...
        com.spawn()
            .insert(tile)
            .insert(GlobalTransform::default())
            .insert(Transform::default())
            .push_children(&children)
            .id()
    }

    fn is_placable(&self, grid: &Grid, coord: &Coordinate) -> bool {
        for &(x, y) in self.geometry.iter() {
            if !(coord.0 + x < GRID_SIZE && coord.1 + y < GRID_SIZE) {
                return false;
            }
            if !grid.is_free(&(coord.0 + x, coord.1 + y)) {
                return false;
            }
        }
        true
    }

    fn try_place(
        &self,
        grid: &mut Grid,
        position: &Coordinate,
    ) -> Result<Vec<Entity>, &'static str> {
        //return an error(?)
        if !self.is_placable(&grid, &position) {
            return Err("Can't place the tile here");
        }
        Ok(self
            .geometry
            .iter()
            .map(|&(x, y)| grid.cultivate(&(position.0 + x, position.1 + y), &self.cultivation))
            .collect())
    }
}

impl Default for Tile {
    fn default() -> Self {
        let geometry = vec![(1, 0), (0, 1), (0, 0)];
        Self {
            geometry,
            cultivation: Cultivation::Village,
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

const ASSETS: [(&'static str, &'static str); 8] = [
    ("mountain", "mountain.png"),
    ("ruin", "ruin.png"),
    ("default", "default.png"),
    ("farm", "farm.png"),
    ("forest", "forest.png"),
    ("goblin", "goblin,png"),
    ("river", "river.png"),
    ("village", "village.png")
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

fn init_assets(
    mut com: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = AssetManager::new(asset_server, materials);
    com.insert_resource(assets);
}

fn init_grid(mut com: Commands, assets: Res<AssetManager>) {
    let grid = Grid::new(&mut com);

    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
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

fn move_tiles(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<(&Tile, &mut Transform)>,
    grid: Res<Grid>,
) {
    // we are sure, that there is only one tile active
    if let Some((tile, mut transform)) = query.iter_mut().last() {
        for event in cursor.iter() {
            //calculate the closest cell
            let mut position = event.position;
            let grid_pos = Grid::screen_to_grid(position);
            if tile.is_placable(&grid, &grid_pos) {
                position = Grid::grid_to_screen(grid_pos);
            }
            // IF CANNOT PLACE => DONT MOVE

            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }
    }
}

