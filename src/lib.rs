#![allow(clippy::type_complexity)]
#![feature(iter_intersperse)]
#![feature(let_chains)]

mod animations;
mod asset_holder;
mod character;
mod helpers;
mod item;
mod npc;
mod object;
mod player;
mod reticle;
mod ui;

#[cfg(test)]
mod tests;

use asset_holder::AssetHolder;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_polyline::PolylinePlugin;
use bevy_rapier3d::prelude::*;
use opensimplex_noise_rs::OpenSimplexNoise;
use reticle::ReticleBrightness;

pub const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();

    // Basic setup
    app.insert_resource(WindowDescriptor {
        title: LAUNCHER_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins);

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default());
        bevy::log::info!("Debug mode enabled");
    } else {
        bevy::log::info!("Debug mode disabled");
    };

    app.init_collection::<AssetHolder>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(PolylinePlugin)
        .add_plugin(object::Plugin)
        .add_plugin(character::Plugin)
        .add_plugin(npc::Plugin)
        .add_plugin(player::Plugin)
        .add_plugin(reticle::Plugin)
        .add_plugin(item::Plugin)
        .add_plugin(ui::Plugin)
        .add_plugin(animations::Plugin)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics);

    app
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands
        .spawn_bundle(Camera3dBundle {
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::FixedVertical(5.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(0.0, 9.0, -6.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Name::new("Camera"))
        .insert(player::PlayerCamera {});

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0)
            .looking_at(Vec3::ZERO + Vec3::Z, Vec3::Z),
        directional_light: DirectionalLight {
            illuminance: 32_000.0,
            shadows_enabled: true,
            ..default()
        },
        ..Default::default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.02,
    });

    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Point Light"));
}

fn setup_physics(
    mut commands: Commands,
    assets: Res<asset_holder::AssetHolder>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    let mut vertices = vec![];
    let mut indices = vec![];
    let floor_size = 32;
    let noise_generator = OpenSimplexNoise::new(Some(883_279_212_983_182_319)); // if not provided, default seed is equal to 0
    let scale = 0.12;
    for i in 0..floor_size {
        for j in 0..floor_size {
            vertices.push((
                [
                    j as f32,
                    noise_generator.eval_2d(j as f64 * scale, i as f64 * scale)
                        as f32,
                    i as f32,
                ],
                [0.0, 1.0, 0.0],
                [1.0, 1.0],
            ));
            if i != 0 && j < (floor_size - 1) {
                indices.push(((i - 1) * floor_size) + j);
                indices.push(((i) * floor_size) + j);
                indices.push(((i - 1) * floor_size) + j + 1);
            }

            if j != 0 && i != 0 {
                indices.push(((i) * floor_size) + j - 1);
                indices.push(((i) * floor_size) + j);
                indices.push(((i - 1) * floor_size) + j);
            }
        }
    }

    let indices = bevy::render::mesh::Indices::U32(indices);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut mesh =
        Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    commands
        // plane
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -1.5, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Bumpy Floor"));

    /* Floor */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(SceneBundle {
            scene: assets.floor.clone(),
            ..default()
        })
        .insert_bundle(SpatialBundle::from(
            Transform::from_xyz(0.0, -2.0, 0.0).with_scale(Vec3::ONE * 0.3),
        ))
        .insert(Name::new("Floor"));

    /* Create the player. */
    commands
        .spawn()
        .insert_bundle(SpatialBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 0.0,
        )))
        .insert_bundle(character::Bundle::default())
        .insert_bundle(player::Bundle::default())
        .insert_bundle(reticle::Bundle {
            reticle: reticle::Reticle {
                brightness: ReticleBrightness::Full,
                enabled: true,
            },
            ..default()
        })
        .insert(Name::new("Player"));

    /* Create an NPC. */
    commands
        .spawn()
        .insert_bundle(SpatialBundle::from_transform(Transform::from_xyz(
            5.0, 0.0, 5.0,
        )))
        .insert_bundle(character::Bundle {
            movement_properties: character::MovementProperties {
                max_speed: 3.0,
                ..Default::default()
            },
            ..character::Bundle::default()
        })
        .insert(npc::Npc { peaceful: true })
        .insert_bundle(reticle::Bundle {
            reticle: reticle::Reticle {
                enabled: true,
                ..default()
            },
            ..reticle::Bundle::default()
        })
        .insert(reticle::ReticleReceiveType::Enemy)
        .insert(Name::new("Friendly"));

    /* Create an obstacle. */
    for x in 0..=1 {
        for z in 0..=1 {
            commands
                .spawn()
                .insert(RigidBody::Dynamic)
                .insert(Collider::cuboid(0.5, 0.5, 0.5))
                .insert_bundle(object::Bundle::default())
                .insert_bundle(SceneBundle {
                    scene: assets.cube.clone(),
                    transform: Transform::from_xyz(
                        2.0 + (x * 2) as f32,
                        0.5,
                        0.0 + (z * 2) as f32,
                    ),
                    ..default()
                })
                .insert(reticle::ReticleReceiveType::Object)
                .insert(Name::new("Obstacle"));
        }
    }
    /* Create a pickup. */
    commands
        .spawn()
        .with_children(|parent| {
            parent.spawn_bundle(SceneBundle {
                scene: assets.sword.clone(),
                transform: Transform::from_xyz(-0.6, 0.0, 0.0),
                ..default()
            });
            parent.spawn().insert(Collider::cuboid(1.0, 0.3, 0.3));
        })
        .insert_bundle(SpatialBundle::from_transform(Transform::from_xyz(
            5.0, 0.0, 5.0,
        )))
        .insert_bundle(item::Bundle {
            collider: Collider::cuboid(1.2, 0.5, 0.5),
            ..item::Bundle::sword()
        })
        .insert(RigidBody::Dynamic)
        .insert(Name::new("Sword"));

    /* Create a pickup. */
    commands
        .spawn()
        .with_children(|parent| {
            parent.spawn_bundle(SceneBundle {
                scene: assets.gun.clone(),
                transform: Transform::from_xyz(-0.6, 0.0, 0.0),
                ..default()
            });
            parent.spawn().insert(Collider::cuboid(1.0, 0.3, 0.3));
        })
        .insert_bundle(SpatialBundle::from_transform(Transform::from_xyz(
            8.0, 0.0, 5.0,
        )))
        .insert_bundle(item::Bundle {
            collider: Collider::cuboid(1.2, 0.5, 0.5),
            ..item::Bundle::gun()
        })
        .insert(RigidBody::Dynamic)
        .insert(Name::new("Gun"));
}
