use bevy::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Island {
    /// Creates a flat island.
    Flat,
    /// Creates an "lump" shape that bulges up in the middle and falls down
    /// at the sides
    Lump,
    /// Uses simplex noise to generate the island, with the given value as the
    /// seed
    Simplex(i64),

    Scale(Vec3, Box<Island>),
    Translate(Vec3, Box<Island>),
    Terrace(f32, Box<Island>),
    Add(Box<Island>, Box<Island>),
    Min(Box<Island>, Box<Island>),
    Max(Box<Island>, Box<Island>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generation {
    /// The number of generated points per unit
    resolution: f32,
}

pub struct Point {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

impl Island {
    pub fn height_at_point(&self, x: f32, z: f32) -> f32 {
        match self {
            Island::Flat => 0.0,
            _ => {
                todo!()
            }
        }
    }

    pub fn generate(
        self,
        generation_type: &Generation,
        width: f32,
        height: f32,
    ) -> (Vec<Point>, Vec<[u32; 3]>) {
        let color = Color::GREEN;
        todo!()
    }
}
