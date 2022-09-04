#![allow(clippy::type_complexity)]
use bevy::{prelude::*, render::camera::ScalingMode, time::Stopwatch};
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct PlayerCharacter;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct CharacterMovementProperties {
    stopped_friction: f32,
    acceleration: f32,
    damping_factor: f32,
    max_speed: f32,
    jump_impulse: f32,
}

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
enum JumpState {
    #[default]
    Normal,
    Charging(Stopwatch),
    JumpPressed(Stopwatch),
}

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct CharacterInput {
    direction: Vec3,
    jump: JumpState,
}

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct MainCamera;

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
    .add_system(player_input)
    .add_system(force_movement.after(player_input))
    .add_system(impluse_movement.after(player_input))
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
        .insert_bundle(SpatialBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
        .insert(Name::new("Floor"));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Velocity::default())
        .insert_bundle(SpatialBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
        .insert_bundle(SceneBundle {
            scene: asset_server.load("sphere/sphere.gltf#Scene0"),
            ..default()
        })
        .insert(PlayerCharacter {})
        .insert(CharacterMovementProperties {
            stopped_friction: 4.0,
            acceleration: 10.0,
            damping_factor: 30.0,
            max_speed: 10.0,
            jump_impulse: 3.0,
        })
        .insert(CharacterInput::default())
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

fn player_input(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_character: Query<(With<PlayerCharacter>, &mut CharacterInput)>,
) {
    if let Some((_, mut character_input)) = player_character.iter_mut().next() {
        // directional
        {
            let up = keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up);
            let down = keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down);
            let left = keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left);
            let right =
                keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right);
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

            character_input.direction = direction;
        }

        // jump
        {
            match character_input.jump.clone() {
                JumpState::Normal => {
                    if keys.pressed(KeyCode::Space) {
                        character_input.jump =
                            JumpState::Charging(Stopwatch::new());
                    }
                }
                JumpState::Charging(mut watch) => {
                    if keys.pressed(KeyCode::Space) {
                        watch.tick(time.delta());
                        character_input.jump = JumpState::Charging(watch);
                    } else if keys.just_released(KeyCode::Space) {
                        character_input.jump = JumpState::JumpPressed(watch);
                    }
                }
                JumpState::JumpPressed(_watch) => {
                    character_input.jump = JumpState::Normal;
                }
            }
        }
    }
}

fn force_movement(
    mut player_character: Query<(
        &CharacterInput,
        &CharacterMovementProperties,
        &Velocity,
        &mut ExternalForce,
        &mut Friction,
    )>,
) {
    fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
        v - v.project_onto(n)
    }
    if let Some((
        character_input,
        character_movement_properties,
        velocity,
        mut external_force,
        mut friction,
    )) = player_character.iter_mut().next()
    {
        let velocity_direction_difference = velocity
            .linvel
            .try_normalize()
            .map(|v| {
                project_onto_plane(character_input.direction, Vec3::Y)
                    - project_onto_plane(v, Vec3::Y)
            })
            .unwrap_or(Vec3::ZERO);

        if character_input.direction != Vec3::ZERO {
            let under_max_speed = velocity
                .linvel
                .project_onto(character_input.direction)
                .length()
                < character_movement_properties.max_speed;
            let directional_force = if under_max_speed {
                character_input.direction
                    * character_movement_properties.acceleration
            } else {
                Vec3::ZERO
            };
            let damping_force = velocity_direction_difference
                * character_movement_properties.damping_factor;
            external_force.force = directional_force + damping_force;
            friction.coefficient = 0.0;
        } else {
            friction.coefficient =
                character_movement_properties.stopped_friction;
            external_force.force = character_input.direction;
        }
    } else {
        println!("No player character found!");
    }
}

fn impluse_movement(
    rapier_context: Res<RapierContext>,
    mut player_character: Query<(
        Entity,
        &CharacterInput,
        &CharacterMovementProperties,
        &Transform,
        &mut ExternalImpulse,
    )>,
) {
    if let Some((
        entity,
        character_input,
        character_movement_properties,
        transform,
        mut external_impulse,
    )) = player_character.iter_mut().next()
    {
        if let Some((_entity, _toi)) = rapier_context.cast_ray(
            // TODO: Should use a shapecast instead
            transform.translation,
            Vec3::NEG_Y,
            1.1,
            true,
            QueryFilter {
                exclude_collider: Some(entity),
                ..default()
            },
        ) {
            if let JumpState::JumpPressed(_watch) = character_input.jump.clone()
            {
                external_impulse.impulse = Vec3::new(
                    0.,
                    character_movement_properties.jump_impulse,
                    0.,
                );
            } else {
                external_impulse.impulse = Vec3::ZERO;
            }
        }
    }
}
