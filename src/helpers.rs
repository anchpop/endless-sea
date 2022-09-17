use bevy::prelude::*;

pub fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
    v - v.project_onto(n)
}
