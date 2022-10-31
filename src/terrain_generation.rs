use bevy::{prelude::*, sprite::Rect};

use crate::helpers::*;

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
    pub point_density: f32,
}

impl Default for Generation {
    fn default() -> Self {
        Self { point_density: 1.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Color,
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
        rect: Rect,
    ) -> (Vec<Point>, Vec<[u32; 3]>) {
        let color = Color::GREEN;
        let num_points = (
            (rect.width() * generation_type.point_density).round() as u32 + 1,
            (rect.height() * generation_type.point_density).round() as u32 + 1,
        );
        let mut points = Vec::new();
        let mut indices = Vec::new();
        for x in 0..num_points.0 {
            for z in 0..num_points.1 {
                let p = {
                    let x_frac = x as f32 / num_points.0 as f32;
                    let z_frac = z as f32 / num_points.1 as f32;
                    let x = lerp(rect.min.x, rect.max.x, x_frac);
                    let z = lerp(rect.min.y, rect.max.y, z_frac);
                    let y = self.height_at_point(x, z);
                    Vec3::new(x as f32, y, z as f32)
                };

                points.push(Point {
                    position: p,
                    normal: Vec3::Y,
                    color,
                });
                if x != 0 && z < (num_points.1 - 1) {
                    indices.push([
                        ((x - 1) * num_points.0) + z,
                        ((x - 1) * num_points.0) + z + 1,
                        ((x) * num_points.0) + z,
                    ]);
                }

                if z != 0 && x != 0 {
                    indices.push([
                        ((x) * num_points.0) + z - 1,
                        ((x - 1) * num_points.0) + z,
                        ((x) * num_points.0) + z,
                    ]);
                }
            }
        }
        (points, indices)
    }
}
