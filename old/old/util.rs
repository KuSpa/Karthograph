use std::convert::TryInto;

use bevy::math::Vec2;

//https://stackoverflow.com/a/29570662/5862030
pub fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn contains_point(pos: &Vec2, size: &Vec2, pointer: &Vec2) -> bool {
    let bounds = *size / Vec2::new(2., 2.);
    !(pointer.x < pos.x - bounds.x
        || pointer.x > pos.x + bounds.x
        || pointer.y < pos.y - bounds.y
        || pointer.y > pos.y + bounds.y)
}

pub fn min_f(lhs: f32, rhs: f32) -> f32 {
    if lhs < rhs {
        return lhs;
    }
    rhs
}
