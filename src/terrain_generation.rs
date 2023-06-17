use bevy::prelude::*;
use opensimplex_noise_rs::OpenSimplexNoise;

use crate::helpers::*;

#[allow(dead_code)]
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
        use Island::*;
        match self {
            Flat => 0.0,
            Simplex(seed) => {
                let noise_generator = OpenSimplexNoise::new(Some(*seed));
                noise_generator.eval_2d(x as f64, z as f64) as f32
            }
            Lump => -(x.powf(2.0) + z.powf(2.0)).sqrt(),
            Scale(scale, island) => {
                let x = x / scale.x;
                let z = z / scale.z;
                island.height_at_point(x, z) * scale.y
            }
            Translate(translation, island) => {
                let x = x - translation.x;
                let z = z - translation.z;
                island.height_at_point(x, z) + translation.y
            }
            Terrace(terrace, island) => {
                (island.height_at_point(x, z) / *terrace).round() * *terrace
            }
            Add(a, b) => a.height_at_point(x, z) + b.height_at_point(x, z),
            Min(a, b) => a.height_at_point(x, z).min(b.height_at_point(x, z)),
            Max(a, b) => a.height_at_point(x, z).max(b.height_at_point(x, z)),
        }
    }

    fn normal_at_point(&self, x: f32, z: f32, dist: f32) -> Vec3 {
        let p = Vec3::new(x, self.height_at_point(x, z), z);
        let adjacents: [(f32, f32); 5] = [
            (dist, 0.0),
            (0.0, dist),
            (-dist, 0.0),
            (0.0, -dist),
            (dist, 0.0),
            // repeating the first one so when we get the windows it
            // wraps around
        ];
        let normal = adjacents
            .windows(2)
            .map(|window| {
                let p1 = {
                    let x = x + window[0].0;
                    let z = z + window[0].1;
                    let y = self.height_at_point(x, z);
                    Vec3::new(x, y, z)
                };
                let p2 = {
                    let x = x + window[1].0;
                    let z = z + window[1].1;
                    let y = self.height_at_point(x, z);
                    Vec3::new(x, y, z)
                };
                let e1 = p - p1;
                let e2 = p - p2;
                e2.cross(e1).normalize_or_zero()
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
                    let position = Vec3::new(x, y, z);
                    let normal = self.normal_at_point(
                        x,
                        z,
                        1.0 / generation_type.vertex_density,
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

    #[allow(dead_code)]
    pub fn scale(self, scale: Vec3) -> Self {
        Island::Scale(scale, Box::new(self))
    }

    #[allow(dead_code)]
    pub fn translate(self, translation: Vec3) -> Self {
        Island::Translate(translation, Box::new(self))
    }

    #[allow(dead_code)]
    pub fn terrace(self, terrace: f32) -> Self {
        Island::Terrace(terrace, Box::new(self))
    }

    #[allow(dead_code)]
    pub fn add(self, other: Self) -> Self {
        Island::Add(Box::new(self), Box::new(other))
    }
}
