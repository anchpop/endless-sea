#![allow(clippy::type_complexity)]

mod character;
mod npc;
#[cfg(test)]
mod tests;

use bevy::{prelude::*, render::camera::ScalingMode, time::Stopwatch};
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct MainCamera;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
struct Reticle;

pub const LAUNCHER_TITLE: &str = "Endless Sea";

pub fn app() -> App {
    let mut app = App::new();

    // Basic setup
    app.add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            title: LAUNCHER_TITLE.to_string(),
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            ..Default::default()
        });

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default());
        bevy::log::info!("Debug mode enabled");
    } else {
        bevy::log::info!("Debug mode disabled");
    };

    static POST_SIMULATION: &str = "post_simulation";
    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(character::Plugin)
        .add_plugin(npc::Plugin)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(player_input)
        .add_system(player_looking_input)
        .add_stage_after(
            PhysicsStages::Writeback,
            POST_SIMULATION,
            SystemStage::parallel(),
        )
        .add_system_to_stage(POST_SIMULATION, camera_movement);

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

    /* Create the player. */
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene: asset_server.load("capsule/capsule.gltf#Scene0"),
            ..default()
        })
        .insert_bundle(character::Bundle::default())
        .insert(character::Player {})
        .insert(Name::new("Player"));

    /* Create the reticle. */
    commands
        .spawn()
        .insert(Reticle {})
        .insert(Name::new("Reticle"));

    /* Create the player. */
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene: asset_server.load("capsule/capsule.gltf#Scene0"),
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
        .insert(Name::new("Friendly"));

    /* Create an obstacle. */
    for x in 0..=1 {
        for z in 0..=1 {
            commands
                .spawn()
                .insert(RigidBody::Dynamic)
                .insert(Collider::cuboid(0.5, 0.5, 0.5))
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(
                        2.0 + x as f32,
                        0.5,
                        0.0 + z as f32,
                    ),
                    ..default()
                })
                .insert(Name::new("Obstacle"));
        }
    }
}

fn camera_movement(
    player_character: Query<(With<character::Player>, &Transform)>,
    mut main_camera: Query<(
        With<MainCamera>,
        Without<character::Player>,
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
    mouse: Res<Input<MouseButton>>,
    mut player_character: Query<(
        With<character::Player>,
        &mut character::Input,
    )>,
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

            character_input.movement_direction = direction;
        }

        // jump
        {
            use character::JumpState::*;
            match character_input.jump.clone() {
                None => {
                    if keys.pressed(KeyCode::Space) {
                        character_input.jump = Some(Charging(Stopwatch::new()));
                    } else if keys.just_released(KeyCode::Space) {
                        character_input.jump =
                            Some(JumpPressed(Stopwatch::new()));
                    }
                }
                Some(Charging(mut watch)) => {
                    if keys.pressed(KeyCode::Space) {
                        watch.tick(time.delta());
                        character_input.jump = Some(Charging(watch));
                    } else if keys.just_released(KeyCode::Space) {
                        character_input.jump = Some(JumpPressed(watch));
                    }
                }
                Some(JumpPressed(_watch)) => {
                    character_input.jump = None;
                }
            }

            // attack
            {
                if mouse.just_pressed(MouseButton::Left) {
                    character_input.attack =
                        Some(character::AttackState::Primary);
                } else if mouse.just_pressed(MouseButton::Right) {
                    character_input.attack =
                        Some(character::AttackState::Secondary);
                } else {
                    character_input.attack = None;
                }
            }
        }
    }
}

fn player_looking_input(
    wnds: Res<Windows>,
    q_camera: Query<(
        &Camera,
        &GlobalTransform,
        With<MainCamera>,
        Without<character::Player>,
    )>,
    mut player_character: Query<(
        With<character::Player>,
        &GlobalTransform,
        &mut character::Input,
    )>,
) {
    if let Some((_, player_transform, mut player_input)) =
        player_character.iter_mut().next()
    {
        if let Some((camera, camera_transform, _, _)) = q_camera.iter().next() {
            // directional
            {
                // get the window that the camera is displaying to (or the
                // primary window)
                let wnd =
                    if let bevy::render::camera::RenderTarget::Window(id) =
                        camera.target
                    {
                        wnds.get(id).unwrap()
                    } else {
                        wnds.get_primary().unwrap()
                    };

                // check if the cursor is inside the window and get its position
                if let Some(cursor_pos_screen_pixels) = wnd.cursor_position() {
                    // get the size of the window
                    let window_size =
                        Vec2::new(wnd.width() as f32, wnd.height() as f32);

                    // Convert screen position [0..resolution] to ndc [-1..1]
                    // (normalized device coordinates)
                    let cursor_ndc = (cursor_pos_screen_pixels / window_size)
                        * 2.0
                        - Vec2::ONE;

                    // matrix for undoing the projection and camera transform
                    let ndc_to_world = camera_transform.compute_matrix()
                        * camera.projection_matrix().inverse();

                    // Use near and far ndc points to generate a ray in world
                    // space. This method is more robust than using the location
                    // of the camera as the start of the ray, because ortho
                    // cameras have a focal point at infinity!
                    let cursor_world_pos_near =
                        ndc_to_world.project_point3(cursor_ndc.extend(-1.0));
                    let cursor_world_pos_far =
                        ndc_to_world.project_point3(cursor_ndc.extend(1.0));

                    // Compute intersection with the player's plane

                    let ray_direction =
                        cursor_world_pos_far - cursor_world_pos_near;

                    let player_plane_normal = player_transform.up();
                    let playr_plane_point = player_transform.translation();

                    let d = ray_direction.dot(player_plane_normal);
                    // if this is false, line is probably parallel to th plane.
                    if d.abs() > 0.0001 {
                        let diff = cursor_world_pos_near - playr_plane_point;
                        let p = diff.dot(player_plane_normal);
                        let dist = p / d;
                        let intersection =
                            cursor_world_pos_near - ray_direction * dist;
                        player_input.looking_direction =
                            intersection - player_transform.translation();
                    }
                }
            }
        }
    }
}
