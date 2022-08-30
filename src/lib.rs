use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Inspectable, Reflect, Component, Default)]
#[reflect(Component)]
struct PlayerCharacter;

#[derive(Inspectable, Reflect, Component, Default)]
#[reflect(Component)]
struct Character;

pub const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(movement);

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new());
        println!("Inspector enabled");
    } else {
        println!("Inspector disabled");
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
            transform: Transform::from_xyz(-6.0, 9.0, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Name::new("Camera"));

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
        .insert(Name::new("Floor"));
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
        let direction = Vec3::new(
            if keys.pressed(KeyCode::W) {
                1.
            } else if keys.pressed(KeyCode::S) {
                -1.
            } else {
                0.
            },
            0.0,
            if keys.pressed(KeyCode::A) {
                -1.
            } else if keys.pressed(KeyCode::D) {
                1.
            } else {
                0.
            },
        )
        .try_normalize();

        if let Some(direction) = direction {
            external_force.force = direction * 10.0;
        } else {
            external_force.force = Vec3::new(0., 0., 0.);
        }
    }
}
