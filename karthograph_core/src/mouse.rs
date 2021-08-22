use bevy::{
    math::Vec2,
    prelude::{EventReader, ResMut},
    window::CursorMoved,
};

#[derive(Default)]
pub struct MousePosition {
    pub inner: Vec2,
}

pub fn mouse_position(mut mouse: ResMut<MousePosition>, mut cursor: EventReader<CursorMoved>) {
    if let Some(event) = cursor.iter().last() {
        mouse.inner = event.position
    };
}
