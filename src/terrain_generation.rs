use bevy::{prelude::*, sprite::Rect};
use opensimplex_noise_rs::OpenSimplexNoise;

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
    pub vertex_density: f32,
}

impl Default for Generation {
    fn default() -> Self {
        Self {
            vertex_density: 1.0,
        }
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
            Island::Simplex(seed) => {
                let noise_generator = OpenSimplexNoise::new(Some(*seed));
                noise_generator.eval_2d(x as f64, z as f64) as f32
            }
            _ => {
                todo!()
            }
        }
    }

    fn normal_at_point(&self, x: f32, z: f32, dist: f32) -> Vec3 {
        let p = Vec3::new(x, self.height_at_point(x, z), z);
        let adjacents: [(i32, i32); 5] = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 0),
            // repeating the first one so when we get the windows it
            // wraps around
        ];
        let normal = adjacents
            .windows(2)
            .map(|window| {
                let p1 = {
                    let x = (x as i32 + window[0].0) as f32;
                    let z = (z as i32 + window[0].1) as f32;
                    let y = self.height_at_point(x, z);
                    Vec3::new(x, y, z)
                };
                let p2 = {
                    let x = (x as i32 + window[1].0) as f32;
                    let z = (z as i32 + window[1].1) as f32;
                    let y = self.height_at_point(x, z);
                    Vec3::new(x, y, z)
                };
                let e1 = p - p1;
                let e2 = p - p2;
                let normal = e2.cross(e1).normalize_or_zero();
                normal
            })
            .fold(Vec3::ZERO, |acc, normal| acc + normal);
        normal / 4.0
    }

    pub fn generate(
        self,
        generation_type: &Generation,
        rect: Rect,
    ) -> (Vec<Point>, Vec<[u32; 3]>) {
        let color = Color::GREEN;
        let num_points = (
            (rect.width() * generation_type.vertex_density).round() as u32 + 1,
            (rect.height() * generation_type.vertex_density).round() as u32 + 1,
        );
        let mut points = Vec::new();
        let mut indices = Vec::new();
        for x in 0..num_points.0 {
            for z in 0..num_points.1 {
                let (position, normal) = {
                    let x_frac = x as f32 / num_points.0 as f32;
                    let z_frac = z as f32 / num_points.1 as f32;
                    let x = lerp(rect.min.x, rect.max.x, x_frac);
                    let z = lerp(rect.min.y, rect.max.y, z_frac);
                    let y = self.height_at_point(x, z);
                    let position = Vec3::new(x as f32, y, z as f32);
                    let normal = self.normal_at_point(
                        x,
                        z,
                        generation_type.vertex_density,
                    );
                    (position, normal)
                };

                points.push(Point {
                    position,
                    normal,
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
