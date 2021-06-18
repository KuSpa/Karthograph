use std::usize;

use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;

const SPRITE_SIZE: f32 = 75.;
const GRID_SIZE: usize = 11;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(init_camera.system())
        .add_startup_system(init_grid.system())
        .run();
}
#[derive(Debug)]
enum OTHER {
    Village,
    River,
    Farm,
    Goblins,
}
#[derive(Debug)]
enum Terrain{
    Normal,
    Mountain,
    Ruin
}

#[derive(Debug)]
struct Field {
    occupied: Option<OTHER>,
    terrain: Terrain,
    x: usize,
    y: usize,
}


#[derive(Debug)]
struct Grid {
    pub entity: Entity,
    pub inner: [[Entity; GRID_SIZE]; GRID_SIZE],
}

impl Grid {

    /// THIS DOES RETURN A PROXY THAT HAS ALL OF ITS FIELDS BEING THE SAME ENTITY
    fn initialize(com: &mut Commands) -> Self {
        let entity = com.spawn().id();
        Grid { entity, inner:  [[entity; GRID_SIZE]; GRID_SIZE]}
    }
}
fn init_camera(mut com: Commands) {
    let mut bundle = OrthographicCameraBundle::new_2d();
    bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    com.spawn_bundle(bundle);
}

fn init_grid(
    mut com: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut grid = Grid::initialize(&mut com);

    let mountains: Vec<(usize, usize)> = vec![(2,2), (3, 9), (5,5),(7,1), (8,8)];//(x,y)
    let ruins: Vec<(usize,usize)> = vec![(1,2), (1,8),(5,1),(5,9), (9,2),(9,8)];

    let mountain_tex = materials.add(asset_server.load("mountain.png").clone().into());
    let normal_tex = materials.add(asset_server.load("default.png").clone().into());
    let ruin_tex =   materials.add(asset_server.load("ruins.png").clone().into());


    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let mut e = com.spawn();
            grid.inner[x][y] = e.id();

            let mat = 
            if mountains.contains(&(x, y)) {
                mountain_tex.clone()
            } else if ruins.contains(&(x, y)) {
                ruin_tex.clone()
            } else {
                normal_tex.clone()
            };

            e.insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(SPRITE_SIZE,SPRITE_SIZE)),
                material: mat,
                transform: Transform::from_xyz((x as f32 + 0.5) * SPRITE_SIZE , (y as f32 + 0.5) * SPRITE_SIZE, 0.),
                ..Default::default()
            });
        }
    }
}
