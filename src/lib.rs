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
mod terrain_generation;
mod ui;

#[cfg(test)]
mod tests;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_wanderlust::WanderlustPlugin;
use bevy_polyline::PolylinePlugin;
use bevy_rapier3d::prelude::*;
use reticle::ReticleBrightness;
use terrain_generation::{Generation, Island};

pub(crate) const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();

    // Basic setup
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: LAUNCHER_TITLE.to_string(),
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            ..Default::default()
        }),
        ..default()
    }));

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default());
        bevy::log::info!("Debug mode enabled");
    } else {
        bevy::log::info!("Debug mode disabled");
    };

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(PolylinePlugin)
        .add_plugin(WanderlustPlugin)
        .add_plugin(asset_holder::Plugin)
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
    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::FixedVertical(5.0),
                ..default()
            }
            .into(),
            transform: Transform::from_translation(
                Vec3::new(0.0, 9.0, -6.0) * 3.0,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        player::PlayerCamera {},
        Name::new("Camera"),
    ));

    commands.spawn(DirectionalLightBundle {
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

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
        Name::new("Point Light"),
    ));
}

fn add_terrain_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    island: Island,
    generation_type: Generation,
    size: f32,
) {
    let (points, indices) = island.generate(
        &generation_type,
        Rect {
            min: Vec2::new(-size / 2.0, -size / 2.0),
            max: Vec2::new(size / 2.0, size / 2.0),
        },
    );

    let mesh = {
        let indices = bevy::render::mesh::Indices::U32(
            indices.iter().cloned().flatten().collect(),
        );
        let positions = points
            .iter()
            .map(|p| [p.position.x, p.position.y, p.position.z])
            .collect::<Vec<_>>();
        let normals = points
            .iter()
            .map(|p| [p.normal.x, p.normal.y, p.normal.z])
            .collect::<Vec<_>>();
        let colors = points
            .iter()
            .map(|p| [p.color.r(), p.color.g(), p.color.b(), p.color.a()])
            .collect::<Vec<_>>();

        let mut mesh =
            Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh
    };

    let mesh = meshes.add(mesh);
    let material = materials.add(StandardMaterial::default());

    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Collider::trimesh(points.iter().map(|p| p.position).collect(), indices),
        Name::new("generated mesh"),
    ));
}

fn setup_physics(
    mut commands: Commands,
    assets: Res<asset_holder::AssetHolder>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use Island::*;
    let islands = [(
        Lump.scale(Vec3::new(2.0, 0.5, 2.0))
            .add(Simplex(0).scale(Vec3::new(3.0, 1.0, 3.0)))
            .translate(Vec3::Y * -2.0),
        Generation {
            vertex_density: 1.0,
        },
        80.0,
    )];

    for island in islands {
        let (island, generation_type, size) = island;
        add_terrain_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            island,
            generation_type,
            size,
        )
    }

    /* Create the player. */
    commands.spawn((
        character::Bundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        player::Bundle::default(),
        reticle::Bundle {
            reticle: reticle::Reticle {
                brightness: ReticleBrightness::Full,
                enabled: true,
            },
            ..default()
        },
        Name::new("Player"),
    ));

    /* Create an NPC. */
    commands.spawn((
        character::Bundle::from_transform(Transform::from_xyz(5.0, 0.0, 5.0)),
        npc::Npc { peaceful: true },
        reticle::Bundle {
            reticle: reticle::Reticle {
                enabled: true,
                ..default()
            },
            ..reticle::Bundle::default()
        },
        reticle::ReticleReceiveType::Enemy,
        Name::new("Friendly"),
    ));

    /* Create an obstacle. */
    for x in 0..=1 {
        for z in 0..=1 {
            commands.spawn((
                object::Bundle::default(),
                RigidBody::Dynamic,
                Collider::cuboid(0.5, 0.5, 0.5),
                SceneBundle {
                    scene: assets.cube.clone(),
                    transform: Transform::from_xyz(
                        2.0 + (x * 2) as f32,
                        0.5,
                        0.0 + (z * 2) as f32,
                    ),
                    ..default()
                },
                reticle::ReticleReceiveType::Object,
                Name::new("Obstacle"),
            ));
        }
    }
    /* Create a pickup. */
    commands
        .spawn((
            SpatialBundle::from_transform(Transform::from_xyz(5.0, 0.0, 5.0)),
            item::Bundle {
                collider: Collider::cuboid(1.0, 0.3, 0.3),
                ..item::Bundle::sword()
            },
            RigidBody::Dynamic,
            Name::new("Sword"),
        ))
        .with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: assets.sword.clone(),
                transform: Transform::from_xyz(-0.6, 0.0, 0.0),
                ..default()
            });
        });

    /* Create a pickup. */
    commands
        .spawn((
            SpatialBundle::from_transform(Transform::from_xyz(8.0, 0.0, 5.0)),
            item::Bundle {
                collider: Collider::cuboid(1.0, 0.3, 0.3),
                ..item::Bundle::gun()
            },
            RigidBody::Dynamic,
            Name::new("Gun"),
        ))
        .with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: assets.gun.clone(),
                transform: Transform::from_xyz(-0.6, 0.0, 0.0),
                ..default()
            });
        });
}
