#![allow(clippy::type_complexity)]
#![feature(iter_intersperse)]
#![feature(let_chains)]

mod assets;
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

use assets::Assets;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_polyline::PolylinePlugin;
use bevy_rapier3d::prelude::*;
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

    app.init_collection::<Assets>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(PolylinePlugin)
        .add_plugin(object::Plugin)
        .add_plugin(character::Plugin)
        .add_plugin(npc::Plugin)
        .add_plugin(player::Plugin)
        .add_plugin(reticle::Plugin)
        .add_plugin(item::Plugin)
        .add_plugin(ui::Plugin)
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

    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Point Light"));
}

fn setup_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<Assets>,
) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(SceneBundle {
            scene: assets.floor.clone(),
            ..default()
        })
        .insert_bundle(SpatialBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
        .insert(Name::new("Floor"));

    /* Create the player. */
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene: assets.character.clone(),
            ..default()
        })
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
        .insert_bundle(SceneBundle {
            scene: assets.character.clone(),
            transform: Transform::from_xyz(5.0, 0.0, 5.0),
            ..default()
        })
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
                        2.0 + x as f32,
                        0.5,
                        0.0 + z as f32,
                    ),
                    ..default()
                })
                .insert(reticle::ReticleReceiveType::Object)
                .insert(Name::new("Obstacle"))
                .insert(object::ExplodeIntoPieces {
                    pieces: (0..4)
                        .map(|i| {
                            (
                                asset_server
                                    .load(&format!("cube/cube.gltf#Scene{i}")),
                                Collider::cuboid(0.1, 0.1, 0.1),
                            )
                        })
                        .collect(),
                });
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
