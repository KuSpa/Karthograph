use bevy::input::mouse::MouseButtonInput;
use bevy::math::IVec2;

use crate::mouse::MousePosition;
use crate::shape::{Geometry, Shape};
use crate::util::contains_point;
use crate::{asset_management::AssetManager, grid::Cultivation};
use crate::{GRID_OFFSET, GRID_SIZE, SPRITE_SIZE};
use bevy::prelude::*;

pub enum Card {
    Splinter(SplinterDefinition),
    Shape(ShapeDefinition),
    Cultivation(CultivationDefinition),
}
#[derive(Clone)]
pub struct ShapeSpawner {
    shape: Shape,
}

impl Card {
    fn new_shape(left: Geometry, right: Geometry, cultivation: Cultivation) -> Self {
        Card::Shape(ShapeDefinition {
            left,
            right,
            cultivation,
        })
    }

    fn new_cultivation(geometry: Geometry, left: Cultivation, right: Cultivation) -> Self {
        Card::Cultivation(CultivationDefinition {
            geometry,
            left,
            right,
        })
    }

    fn spawn(self, com: &mut Commands, assets: &AssetManager) {
        let handle = assets.fetch("blank_card").unwrap(); // TODO MAKE ME SAFE AND SOUND
        let transform = Transform::from_xyz(
            GRID_SIZE as f32 * SPRITE_SIZE + GRID_OFFSET * 2. + 100.,
            300.,
            0.,
        ); //ANKOR IS IN THE MIDDLE
        let entity = com
            .spawn()
            .insert_bundle(SpriteBundle {
                material: handle,
                transform,
                ..Default::default()
            })
            .id();

        match &self {
            Card::Shape(def) => def.spawn(com, entity, &assets),
            Card::Cultivation(def) => def.spawn(com, entity, &assets),
            _ => panic!("Not yet implemented"),
        }
        com.entity(entity).insert(self);
    }
}

impl Default for Card {
    fn default() -> Self {
        let one = Geometry {
            inner: vec![IVec2::new(0, 0), IVec2::new(1, 1)],
        };

        Card::new_cultivation(one, Cultivation::Goblin, Cultivation::Farm)
    }
}

struct ShapeDefinition {
    left: Geometry,
    right: Geometry,
    cultivation: Cultivation,
}

impl ShapeDefinition {
    fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager) {
        let transform = Transform::from_xyz(0., 75., 0.1); // TODO REMOVE MAGIC NUMBERS
        let handle = assets.fetch(self.cultivation.into()).unwrap();
        let mut children: Vec<Entity> = Default::default();
        //Cultivation field
        children.push(
            com.spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(100., 100.)),
                    material: handle,
                    transform,
                    ..Default::default()
                })
                .id(),
        );

        // Geometry fields
        // first

        const DISTANCE: f32 = 50.;
        let normal_handle = assets.fetch("default").unwrap();
        let right_spawner = ShapeSpawner {
            shape: Shape::new(&self.right, &self.cultivation),
        };
        let left_spawner = ShapeSpawner {
            shape: Shape::new(&self.left, &self.cultivation),
        };
        let left_children: Vec<Entity> = self
            .left
            .as_transform(DISTANCE, 0.2)
            .iter()
            .map(|&transform| {
                com.spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(DISTANCE, DISTANCE)),
                        material: normal_handle.clone(),
                        transform,
                        ..Default::default()
                    })
                    .insert(left_spawner.clone()) // If I want an 'AREA' i can add this to the parent entity `left` and add an Rectangle, where it should be clicked...
                    .id()
            })
            .collect();
        // TODO: depending on how large the shape is, one should adapt this transform
        let left_transform = Transform::from_xyz(-80., -100., 0.);
        let left = com
            .spawn()
            .insert(left_transform)
            .insert(GlobalTransform::default())
            .push_children(&left_children)
            .id();
        children.push(left);

        let right_children: Vec<Entity> = self
            .right
            .as_transform(DISTANCE, 0.2)
            .iter()
            .map(|&transform| {
                com.spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(DISTANCE, DISTANCE)),
                        material: normal_handle.clone(),
                        transform,
                        ..Default::default()
                    })
                    .insert(right_spawner.clone())
                    .id()
            })
            .collect();
        let right_transform = Transform::from_xyz(80., -100., 0.);
        let right = com
            .spawn()
            .insert(right_transform)
            .insert(GlobalTransform::default())
            .push_children(&right_children)
            .id();
        children.push(right);

        com.entity(parent).push_children(&children);
    }
}

struct CultivationDefinition {
    geometry: Geometry,
    left: Cultivation,
    right: Cultivation,
}

impl CultivationDefinition {
    pub fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager) {
        let offset = Vec3::new(0., 50., 0.1);
        const DISTANCE: f32 = 50.; //TODO REMOVE MAGIC NUMBERS
        let normal_handle = assets.fetch("default").unwrap();
        let mut children: Vec<Entity> = self
            .geometry
            .as_transform(DISTANCE, 0.0)
            .iter_mut()
            .map(|transform| {
                transform.translation += offset;
                com.spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(DISTANCE, DISTANCE)),
                        transform: *transform,
                        material: normal_handle.clone(),
                        ..Default::default()
                    })
                    .id()
            })
            .collect();
        // Cultivation children
        let left_transform = Transform::from_xyz(-50., -50., 0.1);
        let left_spawn = ShapeSpawner {
            shape: Shape::new(&self.geometry, &self.left),
        };
        let left_mat = assets.fetch(self.left.into()).unwrap();
        children.push(
            com.spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(DISTANCE, DISTANCE)),
                    material: left_mat,
                    transform: left_transform,
                    ..Default::default()
                })
                .insert(left_spawn)
                .id(),
        );

        let right_transform = Transform::from_xyz(50., -50., 0.1);
        let right_spawn = ShapeSpawner {
            shape: Shape::new(&self.geometry, &self.right),
        };
        let right_mat = assets.fetch(self.right.into()).unwrap();
        children.push(
            com.spawn()
                .insert_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(DISTANCE, DISTANCE)),
                    material: right_mat,
                    transform: right_transform,
                    ..Default::default()
                })
                .insert(right_spawn)
                .id(),
        );
        com.entity(parent).push_children(&children);
    }
}

struct SplinterDefinition {/* TODO */}

pub fn spawn_card(mut com: Commands, query: Query<&Card>, assets: Res<AssetManager>) {
    if query.iter().len() == 0 {
        Card::default().spawn(&mut com, &assets);
    }
}

pub fn click_card(
    mut com: Commands,
    query: Query<(&ShapeSpawner, &GlobalTransform, &Sprite)>,
    cards: Query<&Card>,
    shape: Query<(&Shape, Entity)>,
    mut events: EventReader<MouseButtonInput>,
    position: Res<MousePosition>,
    assets: Res<AssetManager>,
) {
    if cards.iter().len() != 1 {
        return;
    } //TODO FIX
    let card = cards.single().unwrap(); // TODO
    for event in events.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            for (shape_spawner, transform, sprite) in query.iter() {
                if contains_point(
                    &transform.translation.truncate(),
                    &sprite.size,
                    &position.inner,
                ) {
                    // remove current Shape
                    if let Ok((_, shape_entity)) = shape.single() {
                        com.entity(shape_entity).despawn_recursive();
                    }
                    let s = shape_spawner.shape.clone();
                    s.spawn(&mut com, &assets);
                    return;
                }
            }
        }
    }
}
