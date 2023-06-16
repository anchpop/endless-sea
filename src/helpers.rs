use bevy::prelude::*;

pub(crate) fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
    v - v.project_onto(n)
}

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub(crate) fn _inverse_lerp(a: f32, b: f32, t: f32) -> f32 {
    (t - a) / (b - a)
}
