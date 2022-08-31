use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Inspectable, Reflect, Component, Default)]
#[reflect(Component)]
struct PlayerCharacter;

#[derive(Inspectable, Reflect, Component, Default)]
#[reflect(Component)]
struct Character;

#[derive(Inspectable, Reflect, Component, Default)]
#[reflect(Component)]
struct MainCamera;

pub const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: LAUNCHER_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_startup_system(setup_graphics)
    .add_startup_system(setup_physics)
    .add_system(movement)
    .add_system(camera_movement);

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default());
        bevy::log::info!("Debug mode enabled");
    } else {
        bevy::log::info!("Debug mode disabled");
    };

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
        .insert(MainCamera {});

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(SceneBundle {
            scene: asset_server.load("floor/floor.gltf#Scene0"),
            ..default()
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            0.0, -2.0, 0.0,
        )))
        .insert(Name::new("Floor"));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            0.0, 4.0, 0.0,
        )))
        .insert_bundle(SceneBundle {
            scene: asset_server.load("sphere/sphere.gltf#Scene0"),
            ..default()
        })
        .insert(PlayerCharacter {})
        .insert(Character {})
        .insert(ExternalForce {
            force: Vec3::new(0., 0., 0.),
            torque: Vec3::new(0., 0., 0.),
        })
        .insert(Name::new("Ball"));

    /* Create an obstacle. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(2.0, 0.5, 0.0),
            ..default()
        })
        .insert(Name::new("Obstacle"));
}

fn camera_movement(
    player_character: Query<(With<PlayerCharacter>, &Transform)>,
    mut main_camera: Query<(
        With<MainCamera>,
        Without<PlayerCharacter>,
        &mut Transform,
    )>,
) {
    if let Some((_, player_transform)) = player_character.iter().next() {
        if let Some((_, _, mut camera_transform)) =
            main_camera.iter_mut().next()
        {
            camera_transform.translation =
                player_transform.translation + Vec3::new(0.0, 9.0, -6.0);
        }
    }
}

fn movement(
    keys: Res<Input<KeyCode>>,
    mut player_character: Query<(
        With<PlayerCharacter>,
        With<Character>,
        &mut ExternalForce,
    )>,
) {
    if let Some((_, _, mut external_force)) = player_character.iter_mut().next()
    {
        let up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
        let down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
        let left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
        let right = keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
        let direction = Vec3::new(
            if left {
                1.
            } else if right {
                -1.
            } else {
                0.
            },
            0.0,
            if up {
                1.
            } else if down {
                -1.
            } else {
                0.
            },
        )
        .try_normalize()
        .unwrap_or(Vec3::ZERO);

        external_force.force = direction * 10.0;
    }
}
