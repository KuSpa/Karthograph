use bevy::{
    ecs::{query::Access, system::Command},
    input::mouse::{MouseButtonInput, MouseWheel},
    prelude::Plugin,
};
use bevy_spicy_networking::NetworkClient;
use kartograph_core::{
    card::{Card, Choice, Rotation},
    grid::{Coordinate, Cultivation, Geometry, GridLike, Shape},
    network::CCommand,
};

use crate::{
    asset_management::{AssetID, AssetManager},
    CGrid, ClientGameState, MousePosition, SPRITE_SIZE,
};
use bevy::prelude::*;

pub struct ShapePlugin;

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(ClientGameState::ActiveTurn)
                .with_system(click_shape.system())
                .with_system(move_shape.system())
                .with_system(mirror_shape.system())
                .with_system(rotate_shape.system())
                .with_system(drop_shape.system()),
        );
    }
}

pub fn spawn_shape(
    shape: Shape,
    com: &mut Commands,
    assets: &Res<AssetManager>,
    choice: Choice,
    transform: Transform,
) -> Entity {
    let handle = assets.fetch(shape.cultivation.asset_id()).unwrap();

    let mut children = Vec::<Entity>::new();
    let mat = handle;
    let parent = com.spawn().id();

    for &transform in shape.geometry.as_transforms(SPRITE_SIZE, 0.).iter() {
        let child = com
            .spawn()
            .insert(ParentShape(parent))
            .insert(Clickable)
            .insert_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                material: mat.clone(),
                transform,
                ..Default::default()
            })
            .id();
        children.push(child);
    }

    com.entity(parent)
        .insert(shape)
        .insert(choice)
        .insert(GlobalTransform::default())
        .insert(transform)
        .push_children(&children)
        .id()
}

struct ParentShape(Entity);

pub struct ShapeArea(Entity);

pub struct ActiveShape {
    pub entity: Entity,
    original_transform: Transform,
    mirror: bool,
    rotation: Rotation,
}

impl ActiveShape {
    pub fn new(e: Entity, tr: Transform) -> Self {
        Self {
            entity: e,
            original_transform: tr,
            mirror: false,
            rotation: Rotation::North,
        }
    }

    pub fn mirror(&mut self) {
        self.mirror = !self.mirror;
    }

    pub fn rotate_cw(&mut self) {
        self.rotation.rotate_cw();
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation.rotate_ccw();
    }

    pub fn reset(&mut self, shape: &mut Shape) -> Box<dyn Fn(&mut Transform) -> ()> {
        let mirr = self.mirror;
        let mut rotations = 0;
        while self.rotation != Rotation::North {
            self.rotate_cw();
            shape.rotate_ck();
            rotations += 1;
        }

        Box::new(move |tr: &mut Transform| {
            if mirr {
                tr.translation.x = -tr.translation.x;
            };
            for _ in 0..rotations {
                rotate_transform_cw(tr);
            }
        })
    }
}

struct Clickable;

fn click_shape(
    mut com: Commands,
    parents: Query<&Parent>,
    mut shapes: Query<&mut Shape>,
    mut transform_q: Query<(&Parent, &mut Transform)>,
    query: Query<(&ParentShape, &GlobalTransform, &Sprite), With<Clickable>>,
    position: Res<MousePosition>,
    mut active_shape: Option<ResMut<ActiveShape>>,
    mouse: Res<Input<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (shape_id, g_transform, sprite) in query.iter() {
            if contains_point(&g_transform.translation.truncate(), &sprite.size, &position) {
                if let Some(ref mut old) = active_shape {
                    if shape_id.0 == old.entity {
                        continue;
                    }

                    //reset old shape
                    let parent = parents.get(shape_id.0).unwrap();
                    let e = old.entity;
                    let reset_transforms = old.reset(&mut shapes.get_mut(e).unwrap());
                    transform_q
                        .iter_mut()
                        .filter_map(
                            |(Parent(ent), tr)| {
                                if *ent == old.entity {
                                    Some(tr)
                                } else {
                                    None
                                }
                            },
                        )
                        .for_each(|mut tr| (reset_transforms)(&mut tr));
                    com.entity(old.entity)
                        .insert(old.original_transform)
                        .insert(Parent(**parent));
                }
                let parent_transform = transform_q.get_mut(shape_id.0).unwrap().1.clone();
                com.insert_resource(ActiveShape::new(shape_id.0, parent_transform));
                com.entity(shape_id.0).remove::<Parent>();
            }
        }
    }
}

fn move_shape(
    curr_shape: Option<Res<ActiveShape>>,
    mut query: Query<(&Shape, &mut Transform)>,
    mouse: Res<MousePosition>,
    grid: Res<CGrid>,
) {
    if let Some(curr) = curr_shape {
        let (shape, mut tr) = query.get_mut(curr.entity).unwrap();

        let mut position = mouse.0;
        let grid_pos = CGrid::screen_to_grid(position);
        if grid.accepts_geometry_at(&shape.geometry, &grid_pos, &shape.ruin) {
            position = CGrid::grid_to_screen(grid_pos);
        }

        tr.translation.x = position.x;
        tr.translation.y = position.y;
    }
}

fn mirror_shape(
    mut active_shape: Option<ResMut<ActiveShape>>,
    clicks: Res<Input<MouseButton>>,
    mut shape: Query<&mut Shape>,
    mut query: Query<(&Parent, &mut Transform)>,
) {
    if clicks.just_pressed(MouseButton::Middle) {
        if let Some(ref mut active) = active_shape {
            query
                .iter_mut()
                .filter_map(|(Parent(ent), tr)| {
                    if *ent == active.entity {
                        Some(tr)
                    } else {
                        None
                    }
                })
                .for_each(|mut tr| tr.translation.x = -tr.translation.x);
            active.mirror();
            shape.get_mut(active.entity).unwrap().mirror();
        }
    }
}

fn rotate_shape(
    mut active_shape: Option<ResMut<ActiveShape>>,
    mut wheel_events: EventReader<MouseWheel>,
    mut shapes: Query<&mut Shape>,
    mut query: Query<(&Parent, &mut Transform)>,
) {
    for event in wheel_events.iter() {
        if let Some(ref mut active) = active_shape {
            let transforms = query.iter_mut().filter_map(|(Parent(ent), tr)| {
                if *ent == active.entity {
                    Some(tr)
                } else {
                    None
                }
            });

            let mut shape = shapes.get_mut(active.entity).unwrap();
            if event.y < 0. {
                // CW
                shape.rotate_ck();
                for mut transform in transforms {
                    rotate_transform_cw(&mut transform);
                }
                active.rotate_cw();
            } else {
                // CCW
                shape.rotate_cck();
                for mut transform in transforms {
                    rotate_transform_ccw(&mut transform);
                }
                active.rotate_ccw();
            }
        }
    }
}

fn drop_shape(
    mut state: ResMut<State<ClientGameState>>,
    net: Res<NetworkClient>,
    active_shape: Option<Res<ActiveShape>>,
    transforms: Query<(&Transform, &Shape, &Choice)>,
    grid: Res<CGrid>,
    clicks: Res<Input<MouseButton>>,
) {
    if clicks.just_pressed(MouseButton::Left) {
        if let Some(ref act) = active_shape {
            let (transform, shape, choice) = transforms.get(act.entity).unwrap();
            let grid_pos =
                CGrid::screen_to_grid(Vec2::new(transform.translation.x, transform.translation.y));
            if grid.accepts_geometry_at(&shape.geometry, &grid_pos, &shape.ruin) {
                info!("Placed Shape at: {:?}. Submitting to server", grid_pos);
                net.send_message(CCommand::Place {
                    choice: *choice,
                    mirror: act.mirror,
                    position: grid_pos,
                    rotation: act.rotation,
                })
                .unwrap();
                info!("{:?} - set to waiting", state.current());
                state.pop().unwrap();
            }
        }
    }
}

fn rotate_transform_cw(transform: &mut Transform) {
    let x = transform.translation.x;
    let y = transform.translation.y;
    transform.translation.x = y;
    transform.translation.y = -x;
}

fn rotate_transform_ccw(transform: &mut Transform) {
    let x = transform.translation.x;
    let y = transform.translation.y;
    transform.translation.x = -y;
    transform.translation.y = x;
}

fn contains_point(pos: &Vec2, size: &Vec2, pointer: &Vec2) -> bool {
    let bounds = *size / Vec2::new(2., 2.);
    !(pointer.x < pos.x - bounds.x
        || pointer.x > pos.x + bounds.x
        || pointer.y < pos.y - bounds.y
        || pointer.y > pos.y + bounds.y)
}
