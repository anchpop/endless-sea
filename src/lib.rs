use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics);
    app
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn setup_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        )));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            0.0, 4.0, 0.0,
        )))
        .insert_bundle(SceneBundle {
            scene: asset_server.load("sphere/sphere.gltf#Scene0"),
            ..default()
        });
}
