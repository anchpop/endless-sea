use bevy::prelude::*;

pub fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
    v - v.project_onto(n)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn _inverse_lerp(a: f32, b: f32, t: f32) -> f32 {
    (t - a) / (b - a)
}
