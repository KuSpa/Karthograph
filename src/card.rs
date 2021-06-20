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

pub enum CardClickable {
    Left,
    Right,
}

impl Card {
    fn new_shape(left: Geometry, right: Geometry, cultivation: Cultivation) -> Self {
        Card::Shape(ShapeDefinition {
            left,
            right,
            cultivation,
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
            Card::Shape(def) => {
                def.spawn(com, entity, &assets);
            }
            _ => panic!("Not yet implemented"),
        }
        com.entity(entity).insert(self);
    }

    fn spawn_shape(
        &self,
        com: &mut Commands,
        selection: &CardClickable,
        assets: &Res<AssetManager>,
    ) {
        match &self {
            &Card::Shape(def) => def.spawn_shape(com, selection, assets),
            _ => panic!("Not yet implemented"),
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        let one = Geometry {
            inner: vec![IVec2::new(0, 0), IVec2::new(1, 1)],
        };
        let sec = Geometry {
            inner: vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)],
        };
        let cultivation = Cultivation::Farm;
        Card::new_shape(one, sec, cultivation)
    }
}

struct ShapeDefinition {
    left: Geometry,
    right: Geometry,
    cultivation: Cultivation,
}

impl ShapeDefinition {
    //add Click handler for the options... thus, they do not need to be registered by the card usw....
    fn spawn(&self, com: &mut Commands, parent: Entity, assets: &AssetManager) {
        let transform = Transform::from_xyz(0., 75., 0.1); // TODO REMOVE MAGIC NUMBERS
        let handle = assets.fetch(self.cultivation.into()).unwrap();
        let mut children: Vec<Entity> = Default::default();
        //Cultivation field
        children.push(
            com.spawn()
                .insert_bundle(SpriteBundle {
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
                    .insert(CardClickable::Left) // If I want an 'AREA' i can add this to the parent entity `left` and add an Rectangle, where it should be clicked...
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
                    .insert(CardClickable::Right)
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

    fn spawn_shape(
        &self,
        com: &mut Commands,
        selection: &CardClickable,
        assets: &Res<AssetManager>,
    ) {
        println!("spawn");
        let geom = match &selection {
            CardClickable::Left => &self.left,
            CardClickable::Right => &self.right,
        };
        let shape = Shape::new(&geom, &self.cultivation);
        shape.spawn(com, &assets);
    }
}

struct CultivationDefinition {/* TODO */}
struct SplinterDefinition {/* TODO */}

pub fn spawn_card(mut com: Commands, query: Query<&Card>, assets: Res<AssetManager>) {
    if query.iter().len() == 0 {
        Card::default().spawn(&mut com, &assets);
    }
}

pub fn click_card(
    mut com: Commands,
    query: Query<(&CardClickable, &GlobalTransform, &Sprite)>,
    cards: Query<&Card>,
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
            for (side, transform, sprite) in query.iter() {
                if contains_point(
                    &transform.translation.truncate(),
                    &sprite.size,
                    &position.inner,
                ) {
                    card.spawn_shape(&mut com, side, &assets);
                }
            }
        }
    }
}
