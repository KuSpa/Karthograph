use crate::asset_management::AssetManager;
use crate::grid::{Coordinate, Cultivation, FieldComponent, Grid};
use crate::SPRITE_SIZE;
use bevy::core::Timer;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
//TODO MIRROR
pub struct Shape {
    geometry: Vec<Coordinate>,
    cultivation: Cultivation,
}

impl Shape {
    // TODO make this non static
    fn spawn(self, com: &mut Commands, handle: Handle<ColorMaterial>) -> Entity {
        //TODO PROPER TEXTURE
        let mut children = Vec::<Entity>::new();
        let mat = handle;

        for &position in self.geometry.iter() {
            let child = com
                .spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    material: mat.clone(),
                    transform: Transform::from_xyz(
                        position.x as f32 * SPRITE_SIZE,
                        position.y as f32 * SPRITE_SIZE,
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
            .insert(self)
            .insert(GlobalTransform::default())
            .insert(Transform::default())
            .push_children(&children)
            .id()
    }

    // POSSIBLE BREAK -> CALLER DETERMINES WHAT TRANSFORMS TO ROTATE
    pub fn rotate_clockwise<'a>(&mut self, transforms: &mut [Mut<Transform>]) {
        for transform in transforms.iter_mut() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            transform.translation.x = y;
            transform.translation.y = -x;
        }
        for position in self.geometry.iter_mut(){
            *position = position.perp().perp().perp();
        }
    }

    // POSSIBLE BREAK -> CALLER DETERMINES WHAT TRANSFORMS TO ROTATE
    pub fn rotate_counter_clockwise<'a>(&mut self, transforms: &mut [Mut<Transform>]) {
        for transform in transforms.iter_mut() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            transform.translation.x = -y;
            transform.translation.y = x;
        }
        for position in self.geometry.iter_mut(){
            *position = position.perp();
        }
    }

    fn is_placable(&self, grid: &Grid, coord: &Coordinate) -> bool {
        for &pos in self.geometry.iter() {
            if !grid.is_free(&(pos+*coord)) {
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
            return Err("Can't place the shape here");
        }
        Ok(self
            .geometry
            .iter()
            .map(|&pos| grid.cultivate(&(pos+*position), &self.cultivation))
            .collect())
    }
}

impl Default for Shape {
    fn default() -> Self {
        let geometry = vec![IVec2::new(1, 0), IVec2::new(0, 1), IVec2::new(0, 0)];
        Self {
            geometry,
            cultivation: Cultivation::Village,
        }
    }
}

pub fn move_shape(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<(&Shape, &mut Transform)>,
    grid: Res<Grid>,
) {
    // we are sure, that there is only one shape active
    if let Ok((shape, mut transform)) = query.single_mut() {
        for event in cursor.iter() {
            //calculate the closest cell
            let mut position = event.position;
            let grid_pos = Grid::screen_to_grid(position);
            if shape.is_placable(&grid, &grid_pos) {
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

// TODO THIS SHOULD BE REMOVED AS CARDS WILL SPAWN THEM LATER ON
pub fn spawn_shape(
    mut com: Commands,
    query: Query<&Shape>,
    mut timer: Local<Timer>,
    time: ResMut<Time>,
    assets: Res<AssetManager>,
) {
    timer.tick(time.delta());

    if query.iter().len() == 0 {
        if timer.just_finished() {
            let shape = Shape::default();
            Shape::default().spawn(&mut com, assets.fetch(shape.cultivation.into()).unwrap());
        } else if timer.finished() {
            timer.reset();
        }
    }
}

pub fn place_shape(
    mut com: Commands,
    shapes: Query<(Entity, &Shape, &Transform)>,
    mut grid: ResMut<Grid>,
    mut clicks: EventReader<MouseButtonInput>,
    assets: Res<AssetManager>,
    mut fields: Query<(&FieldComponent, &mut Handle<ColorMaterial>)>,
) {
    for event in clicks.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            if let Ok((t_entity, shape, transform)) = shapes.single() {
                let position = Vec2::new(transform.translation.x, transform.translation.y);
                let grid_position = Grid::screen_to_grid(position);
                if let Ok(entities) = shape.try_place(&mut grid, &grid_position) {
                    // Well we got through all the ifs and if lets, it's time to DO SOME STUFF
                    for &entity in entities.iter() {
                        let (_, mut handle) = fields.get_mut(entity).unwrap();
                        *handle = assets.fetch(shape.cultivation.into()).unwrap();
                    }
                    com.entity(t_entity).despawn_recursive();
                }
            }
        }
    }
}
