use crate::character;

use bevy::{prelude::*, time::Stopwatch};
use bevy_inspector_egui::Inspectable;
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Player;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct PlayerCamera;

// Resources
// =========

/// Simple resource to store the ID of the connected gamepad.
/// We need to know which gamepad to use for player input.
struct PrimaryGamepad(Gamepad);

// Plugin
// ======
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        static POST_SIMULATION: &str = "post_simulation";
        app.add_system(gamepad_connections)
            .add_system(player_input)
            .add_system(player_looking_input)
            .add_stage_after(
                PhysicsStages::Writeback,
                POST_SIMULATION,
                SystemStage::parallel(),
            )
            .add_system_to_stage(POST_SIMULATION, camera_movement);
    }
}

fn camera_movement(
    player_character: Query<(With<Player>, &Transform)>,
    mut main_camera: Query<(
        With<PlayerCamera>,
        Without<Player>,
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
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    primary_gamepad: Option<Res<PrimaryGamepad>>,
    mut player_character: Query<(With<Player>, &mut character::Input)>,
) {
    if let Some((_, mut character_input)) = player_character.iter_mut().next() {
        // directional
        let direction_keys = {
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

            direction
        };

        // jump
        let jump_state_keys = {
            use character::JumpState::*;
            match character_input.jump.clone() {
                None => {
                    if keys.pressed(KeyCode::Space) {
                        Some(Charging(Stopwatch::new()))
                    } else if keys.just_released(KeyCode::Space) {
                        Some(JumpPressed(Stopwatch::new()))
                    } else {
                        character_input.jump.clone()
                    }
                }
                Some(Charging(mut watch)) => {
                    if keys.pressed(KeyCode::Space) {
                        watch.tick(time.delta());
                        Some(Charging(watch))
                    } else if keys.just_released(KeyCode::Space) {
                        Some(JumpPressed(watch))
                    } else {
                        character_input.jump.clone()
                    }
                }
                Some(JumpPressed(_watch)) => None,
            }
        };

        // attack
        let attack_keys = {
            if mouse.just_pressed(MouseButton::Left) {
                Some(character::AttackState::Primary)
            } else if mouse.just_pressed(MouseButton::Right) {
                Some(character::AttackState::Secondary)
            } else {
                None
            }
        };

        let (direction, jump_state, attack) = {
            if let Some(gamepad) = primary_gamepad {
                // a gamepad is connected, we have the id
                let gamepad = gamepad.0;
                let direction_joystick = {
                    // The joysticks are represented using a separate axis for X
                    // and Y
                    let axis_lx = GamepadAxis {
                        gamepad,
                        axis_type: GamepadAxisType::LeftStickX,
                    };
                    let axis_ly = GamepadAxis {
                        gamepad,
                        axis_type: GamepadAxisType::LeftStickY,
                    };

                    if let (Some(x), Some(z)) =
                        (axes.get(axis_lx), axes.get(axis_ly))
                    {
                        Vec3::new(-x, 0.0, z)
                    } else {
                        Vec3::ZERO
                    }
                };

                let _jump_button = GamepadButton {
                    gamepad,
                    button_type: GamepadButtonType::East,
                };

                let shoot_primary = GamepadButton {
                    gamepad,
                    button_type: GamepadButtonType::RightTrigger2,
                };
                let shoot_secondary = GamepadButton {
                    gamepad,
                    button_type: GamepadButtonType::RightTrigger,
                };
                let gamepad_attack = if buttons.just_pressed(shoot_primary) {
                    Some(character::AttackState::Primary)
                } else if buttons.just_pressed(shoot_secondary) {
                    Some(character::AttackState::Secondary)
                } else {
                    None
                };

                (
                    direction_keys + direction_joystick,
                    jump_state_keys,
                    attack_keys.max(gamepad_attack),
                )
            } else {
                (direction_keys, jump_state_keys, attack_keys)
            }
        };

        character_input.movement_direction = direction;
        character_input.jump = jump_state;
        character_input.attack = attack;
    }
}

fn player_looking_input(
    wnds: Res<Windows>,
    q_camera: Query<(
        &Camera,
        &GlobalTransform,
        With<PlayerCamera>,
        Without<Player>,
    )>,
    mut player_character: Query<(
        With<Player>,
        &GlobalTransform,
        &mut character::Input,
    )>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    primary_gamepad: Option<Res<PrimaryGamepad>>,
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
                let look_direction_mouse =
                    if let Some(cursor_pos_screen_pixels) =
                        wnd.cursor_position()
                    {
                        // get the size of the window
                        let window_size =
                            Vec2::new(wnd.width() as f32, wnd.height() as f32);

                        // Convert screen position [0..resolution] to ndc
                        // [-1..1] (normalized device
                        // coordinates)
                        let cursor_ndc =
                            (cursor_pos_screen_pixels / window_size) * 2.0
                                - Vec2::ONE;

                        // matrix for undoing the projection and camera
                        // transform
                        let ndc_to_world = camera_transform.compute_matrix()
                            * camera.projection_matrix().inverse();

                        // Use near and far ndc points to generate a ray in
                        // world space. This method is
                        // more robust than using the location
                        // of the camera as the start of the ray, because ortho
                        // cameras have a focal point at infinity!
                        let cursor_world_pos_near = ndc_to_world
                            .project_point3(cursor_ndc.extend(-1.0));
                        let cursor_world_pos_far =
                            ndc_to_world.project_point3(cursor_ndc.extend(1.0));

                        // Compute intersection with the player's plane

                        let ray_direction =
                            cursor_world_pos_far - cursor_world_pos_near;

                        let player_plane_normal = player_transform.up();
                        let playr_plane_point = player_transform.translation();

                        let d = ray_direction.dot(player_plane_normal);
                        // if this is false, line is probably parallel to th
                        // plane.
                        if d.abs() > 0.0001 {
                            let diff =
                                cursor_world_pos_near - playr_plane_point;
                            let p = diff.dot(player_plane_normal);
                            let dist = p / d;
                            let intersection =
                                cursor_world_pos_near - ray_direction * dist;
                            Some(intersection - player_transform.translation())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                let look_direction_gamepad = {
                    if let Some(gamepad) = primary_gamepad {
                        // a gamepad is connected, we have the id
                        let gamepad = gamepad.0;
                        let direction_joystick = {
                            // The joysticks are represented using a separate
                            // axis for X and Y
                            let axis_lx = GamepadAxis {
                                gamepad,
                                axis_type: GamepadAxisType::RightStickX,
                            };
                            let axis_ly = GamepadAxis {
                                gamepad,
                                axis_type: GamepadAxisType::RightStickY,
                            };

                            if let (Some(x), Some(z)) =
                                (axes.get(axis_lx), axes.get(axis_ly))
                            {
                                Vec3::new(-x, 0.0, z)
                            } else {
                                Vec3::ZERO
                            }
                        };

                        Some(direction_joystick)
                    } else {
                        None
                    }
                };
                if let Some(look_direction) = look_direction_mouse
                    .or(look_direction_gamepad)
                    .filter(|v| v.clone() != Vec3::ZERO)
                {
                    player_input.looking_direction = look_direction;
                }
            }
        }
    }
}

fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<PrimaryGamepad>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.iter() {
        // the ID of the gamepad
        let id = ev.gamepad;
        match ev.event_type {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(PrimaryGamepad(id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                // if it's the one we previously associated with the player,
                // disassociate it:
                if let Some(PrimaryGamepad(old_id)) = my_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<PrimaryGamepad>();
                    }
                }
            }
            // other events are irrelevant
            _ => {}
        }
    }
}
