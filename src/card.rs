use bevy::input::mouse::MouseButtonInput;
use bevy::math::IVec2;
use serde::Deserialize;

use crate::mouse::MousePosition;
use crate::shape::{Geometry, Shape};
use crate::util::contains_point;
use crate::{asset_management::AssetManager, grid::Cultivation};
use crate::{GRID_OFFSET, GRID_SIZE, SPRITE_SIZE};
use bevy::prelude::*;

// TODO: remove pub, as it does not need to be visible beyond this module
#[derive(Deserialize, Clone)]
pub enum Card {
    // TODO: rename Definitions to Strategies? bc. StrategyPattern?
    Splinter(SplinterDefinition),
    Shape(ShapeDefinition),
    Cultivation(CultivationDefinition),
    Ruin(RuinDefinition),
}
#[derive(Clone)]
pub enum CardClickEvent {
    SpawnShape(Shape),
    Ruin,
}

#[derive(Default)]
pub struct RuinIndicator {
    inner: bool,
}

impl RuinIndicator {
    pub fn set(&mut self) {
        self.inner = true;
    }

    pub fn reset(&mut self) {
        self.inner = false;
    }

    pub fn value(&self) -> bool {
        self.inner
    }
}

impl Card {
    pub fn spawn(self, com: &mut Commands, assets: &AssetManager, ruin: bool) {
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
            Card::Shape(def) => def.spawn(com, entity, &assets, ruin),
            Card::Cultivation(def) => def.spawn(com, entity, &assets, ruin),
            Card::Splinter(def) => def.spawn(com, entity, &assets, ruin),
            Card::Ruin(def) => def.spawn(com, entity, &assets),
        }
        com.entity(entity).insert(self);
    }
}

impl Default for Card {
    fn default() -> Self {
        Card::Splinter(SplinterDefinition)
    }
}

#[derive(Deserialize, Clone)]
pub struct RuinDefinition;
impl RuinDefinition {
    fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager) {
        // Ruins don't do anything, move on, as soon as anything is clicked
        // Make it look nice tho
        let handle = assets.fetch("ruin").unwrap(); //wrap "ruin" into some constant?
        let ent = com
            .spawn()
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(100., 100.)),
                material: handle,
                transform: Transform::from_xyz(0., 0., 0.1),
                ..Default::default()
            })
            .id();
        com.entity(parent)
            .push_children(&[ent])
            .insert(CardClickEvent::Ruin);
    }
}

#[derive(Deserialize, Clone)]
pub struct ShapeDefinition {
    left: Geometry,
    right: Geometry,
    cultivation: Cultivation,
}

impl ShapeDefinition {
    fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager, ruin: bool) {
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
        let right_spawner =
            CardClickEvent::SpawnShape(Shape::new(&self.right, &self.cultivation, ruin));
        let left_spawner =
            CardClickEvent::SpawnShape(Shape::new(&self.left, &self.cultivation, ruin));
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
#[derive(Deserialize, Clone)]
pub struct CultivationDefinition {
    geometry: Geometry,
    left: Cultivation,
    right: Cultivation,
}

impl CultivationDefinition {
    pub fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager, ruin: bool) {
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
        let left_spawn = CardClickEvent::SpawnShape(Shape::new(&self.geometry, &self.left, ruin));
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
        let right_spawn = CardClickEvent::SpawnShape(Shape::new(&self.geometry, &self.right, ruin));
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
#[derive(Deserialize, Clone)]
pub struct SplinterDefinition;
impl SplinterDefinition {
    pub fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager, ruin: bool) {
        // we just have a 5 choice Cultivation card with a geometry of [(0,0)]
        let geom = Geometry {
            inner: vec![IVec2::new(0, 0)],
        };
        // TODO: remove magic numbers
        const SPLINTER_OFFSET: f32 = 75.;
        let shapes = vec![
            (
                Shape::new(&geom, &Cultivation::Farm, ruin),
                Transform::from_xyz(SPLINTER_OFFSET, SPLINTER_OFFSET, 0.1),
            ),
            (
                Shape::new(&geom, &Cultivation::Goblin, ruin),
                Transform::from_xyz(SPLINTER_OFFSET, -SPLINTER_OFFSET, 0.1),
            ),
            (
                Shape::new(&geom, &Cultivation::Water, ruin),
                Transform::from_xyz(-SPLINTER_OFFSET, SPLINTER_OFFSET, 0.1),
            ),
            (
                Shape::new(&geom, &Cultivation::Village, ruin),
                Transform::from_xyz(-SPLINTER_OFFSET, -SPLINTER_OFFSET, 0.1),
            ),
            (
                Shape::new(&geom, &Cultivation::Forest, ruin),
                Transform::from_xyz(0., 0., 0.1),
            ),
        ];
        let children: Vec<Entity> = shapes
            .iter()
            .map(|(shape, transform)| {
                let material = assets.fetch(shape.cultivation.into()).unwrap();
                com.spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(50., 50.)),
                        material,
                        transform: *transform,
                        ..Default::default()
                    })
                    .insert(CardClickEvent::SpawnShape(shape.clone()))
                    .id()
            })
            .collect();
        com.entity(parent).push_children(&children);
    }
}

pub fn click_card(
    mut com: Commands,
    query: Query<(&CardClickEvent, &GlobalTransform, &Sprite, Entity)>,
    shape: Query<(&Shape, Entity)>,
    mut events: EventReader<MouseButtonInput>,
    mut ruin: ResMut<RuinIndicator>,
    position: Res<MousePosition>,
    assets: Res<AssetManager>,
) {
    for event in events.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            for (shape_spawner, transform, sprite, entity) in query.iter() {
                if contains_point(
                    &transform.translation.truncate(),
                    &sprite.size,
                    &position.inner,
                ) {
                    // remove current Shape
                    if let Ok((_, shape_entity)) = shape.single() {
                        com.entity(shape_entity).despawn_recursive();
                    }
                    //let s = shape_spawner.shape.clone();
                    match &shape_spawner {
                        CardClickEvent::SpawnShape(shape) => {
                            shape.clone().spawn(&mut com, &assets);
                        }
                        CardClickEvent::Ruin => {
                            com.entity(entity).despawn_recursive();
                            ruin.set()
                        }
                    };
                    return;
                }
            }
        }
    }
}
