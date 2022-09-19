use bevy::prelude::*;
use bevy_polyline::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::character;

// Components
// ==========

#[allow(dead_code)]
#[derive(Component, Clone)]
pub enum ReticleReceiveType {
    Player,
    Enemy,
    Friendly,
    Object,
}

#[derive(Component, Clone)]
pub struct ReticleEmitColor(pub bool);

// Bundle
// ======

#[derive(Bundle)]
pub struct Bundle {
    pub polyline_material: Handle<PolylineMaterial>,
    pub polyline: Handle<Polyline>,
    pub reticle_emit_color: ReticleEmitColor,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            polyline_material: Handle::<PolylineMaterial>::default(),
            polyline: Handle::<Polyline>::default(),
            reticle_emit_color: ReticleEmitColor(false),
        }
    }
}

// Resources
// =========

struct ReticleMaterials {
    player: Handle<PolylineMaterial>,
    enemy: Handle<PolylineMaterial>,
    friendly: Handle<PolylineMaterial>,
    object: Handle<PolylineMaterial>,
    default: Handle<PolylineMaterial>,
    no_color: Handle<PolylineMaterial>,
}

// Plugin
// ======

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_reticle_materials)
            .add_system(draw_reticle);
    }
}

fn draw_reticle(
    mut reticles: Query<(
        Entity,
        &GlobalTransform,
        &character::Input,
        &ReticleEmitColor,
        &mut Handle<PolylineMaterial>,
        &mut Handle<Polyline>,
    )>,
    receiver: Query<&ReticleReceiveType>,
    polyline_materials: Res<ReticleMaterials>,
    mut polylines: ResMut<Assets<Polyline>>,
    rapier_context: Res<RapierContext>,
) {
    for (
        entity,
        transform,
        input,
        reticle_emit_color,
        mut material,
        mut line,
    ) in reticles.iter_mut()
    {
        if let Some(dir) = input.looking_direction.try_normalize() {
            let (color, distance) = {
                if let Some((entity, toi)) = rapier_context.cast_ray(
                    transform.translation(),
                    dir,
                    1000.0,
                    true,
                    QueryFilter {
                        exclude_collider: Some(entity),
                        ..default()
                    },
                ) {
                    (
                        if reticle_emit_color.0 {
                            if let Ok(receiver) = receiver.get(entity) {
                                match receiver {
                                    ReticleReceiveType::Player => {
                                        polyline_materials.player.clone()
                                    }
                                    ReticleReceiveType::Enemy => {
                                        polyline_materials.enemy.clone()
                                    }
                                    ReticleReceiveType::Friendly => {
                                        polyline_materials.friendly.clone()
                                    }
                                    ReticleReceiveType::Object => {
                                        polyline_materials.object.clone()
                                    }
                                }
                            } else {
                                polyline_materials.default.clone()
                            }
                        } else {
                            polyline_materials.no_color.clone()
                        },
                        toi,
                    )
                } else {
                    (
                        if reticle_emit_color.0 {
                            polyline_materials.default.clone()
                        } else {
                            polyline_materials.no_color.clone()
                        },
                        1000.0,
                    )
                }
            };
            *material = color;
            *line = polylines.add(Polyline {
                vertices: vec![Vec3::ZERO, Vec3::NEG_Z * distance],
            });
        }
    }
}

fn setup_reticle_materials(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let resource = ReticleMaterials {
        player: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::RED,
            perspective: true,
            ..Default::default()
        }),
        enemy: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::RED,
            perspective: true,
            ..Default::default()
        }),
        friendly: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::RED,
            perspective: true,
            ..Default::default()
        }),
        object: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::Rgba {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 1.,
            },
            perspective: true,
            ..Default::default()
        }),
        default: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::Rgba {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 0.5,
            },
            perspective: true,
            ..Default::default()
        }),
        no_color: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::Rgba {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 0.3,
            },
            perspective: true,
            ..Default::default()
        }),
    };
    commands.insert_resource(resource);
}
