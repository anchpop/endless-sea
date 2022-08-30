use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct PlayerCharacter;
#[derive(Component)]
struct Character;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(movement);
    app
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-1.0, 10.0, 0.0)
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
        });
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
