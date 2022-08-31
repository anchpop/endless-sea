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
struct MainCamera {
    velocity: Vec3,
}

pub const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();

    static POST_SIMULATION: &str = "debug";
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
    .add_system(force_movement)
    .add_system(impluse_movement)
    .add_stage_after(
        PhysicsStages::Writeback,
        POST_SIMULATION,
        SystemStage::parallel(),
    )
    .add_system_to_stage(POST_SIMULATION, camera_movement);

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
        .insert(MainCamera {
            velocity: Vec3::ZERO,
        });

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
        .insert_bundle(SpatialBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
        .insert(Name::new("Floor"));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert_bundle(SpatialBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
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
        .insert(ExternalImpulse {
            impulse: Vec3::new(0., 0., 0.),
            torque_impulse: Vec3::new(0., 0., 0.),
        })
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Max,
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
    time: Res<Time>,
    player_character: Query<(With<PlayerCharacter>, &Transform)>,
    mut main_camera: Query<(
        &mut MainCamera,
        Without<PlayerCharacter>,
        &mut Transform,
    )>,
) {
    if let Some((_, player_transform)) = player_character.iter().next() {
        if let Some((mut main_camera, _, mut camera_transform)) =
            main_camera.iter_mut().next()
        {
            let destination =
                player_transform.translation + Vec3::new(0.0, 9.0, -6.0);
            camera_transform.translation = smooth_damp(
                camera_transform.translation,
                destination,
                &mut main_camera.velocity,
                0.1,
                f32::INFINITY,
                time.delta_seconds(),
            )
        }
    }
}

fn force_movement(
    keys: Res<Input<KeyCode>>,
    mut player_character: Query<(
        With<PlayerCharacter>,
        With<Character>,
        &mut ExternalForce,
        &mut Friction,
    )>,
) {
    if let Some((_, _, mut external_force, mut friction)) =
        player_character.iter_mut().next()
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

        if direction == Vec3::ZERO {
            friction.coefficient = 4.0;
        } else {
            friction.coefficient = 0.0;
        }
    }
}

fn impluse_movement(
    keys: Res<Input<KeyCode>>,
    mut player_character: Query<(
        With<PlayerCharacter>,
        With<Character>,
        &mut ExternalImpulse,
    )>,
) {
    if let Some((_, _, mut external_impulse)) =
        player_character.iter_mut().next()
    {
        let jump = keys.just_released(KeyCode::Space);

        external_impulse.impulse =
            Vec3::new(0., if jump { 3. } else { 0. }, 0.);
    }
}

// Gradually changes a vector towards a desired goal over time.
// inspired by https://github.com/Unity-Technologies/UnityCsReference/blob/master/Runtime/Export/Math/Vector3.cs#L97
fn smooth_damp(
    current: Vec3,
    mut target: Vec3,
    current_velocity: &mut Vec3,
    smooth_time: f32,
    max_speed: f32,
    delta_seconds: f32,
) -> Vec3 {
    // Based on Game Programming Gems 4 Chapter 1.10
    let smooth_time = f32::max(0.0001, smooth_time);
    let omega = 2.0 / smooth_time;

    let x = omega * delta_seconds;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let mut change_x = current.x - target.x;
    let mut change_y = current.y - target.y;
    let mut change_z = current.z - target.z;
    let original_to = target;

    // Clamp maximum speed
    let max_change = max_speed * smooth_time;

    let max_change_sq = max_change * max_change;
    let sqrmag =
        change_x * change_x + change_y * change_y + change_z * change_z;
    if sqrmag > max_change_sq {
        let mag = f32::sqrt(sqrmag);
        change_x = change_x / mag * max_change;
        change_y = change_y / mag * max_change;
        change_z = change_z / mag * max_change;
    }

    target.x = current.x - change_x;
    target.y = current.y - change_y;
    target.z = current.z - change_z;

    let temp_x = (current_velocity.x + omega * change_x) * delta_seconds;
    let temp_y = (current_velocity.y + omega * change_y) * delta_seconds;
    let temp_z = (current_velocity.z + omega * change_z) * delta_seconds;

    current_velocity.x = (current_velocity.x - omega * temp_x) * exp;
    current_velocity.y = (current_velocity.y - omega * temp_y) * exp;
    current_velocity.z = (current_velocity.z - omega * temp_z) * exp;

    let mut output_x = target.x + (change_x + temp_x) * exp;
    let mut output_y = target.y + (change_y + temp_y) * exp;
    let mut output_z = target.z + (change_z + temp_z) * exp;

    // Prevent overshooting
    let orig_minus_current_x = original_to.x - current.x;
    let orig_minus_current_y = original_to.y - current.y;
    let orig_minus_current_z = original_to.z - current.z;
    let out_minus_orig_x = output_x - original_to.x;
    let out_minus_orig_y = output_y - original_to.y;
    let out_minus_orig_z = output_z - original_to.z;

    if orig_minus_current_x * out_minus_orig_x
        + orig_minus_current_y * out_minus_orig_y
        + orig_minus_current_z * out_minus_orig_z
        > 0.0
    {
        output_x = original_to.x;
        output_y = original_to.y;
        output_z = original_to.z;

        current_velocity.x = (output_x - original_to.x) / delta_seconds;
        current_velocity.y = (output_y - original_to.y) / delta_seconds;
        current_velocity.z = (output_z - original_to.z) / delta_seconds;
    }

    Vec3::new(output_x, output_y, output_z)
}
