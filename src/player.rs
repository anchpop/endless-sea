use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_inspector_egui::Inspectable;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::{prelude::*, Actionlike};

use crate::character::{self, CanPickUpItems};

// Components
// ==========

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Player;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct PlayerCamera;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Jump,
    Look,
    ShootPrimary,
    ShootSecondary,
}

// Bundle
// ======

#[derive(Bundle)]
pub struct Bundle {
    player: Player,
    action_state: ActionState<Action>,
    input_map: InputMap<Action>,
    can_pick_up_items: CanPickUpItems,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            player: Player::default(),
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(
                    VirtualDPad {
                        up: KeyCode::W.into(),
                        down: KeyCode::S.into(),
                        left: KeyCode::A.into(),
                        right: KeyCode::D.into(),
                    },
                    Action::Move,
                )
                .insert(
                    VirtualDPad {
                        up: KeyCode::Up.into(),
                        down: KeyCode::Down.into(),
                        left: KeyCode::Left.into(),
                        right: KeyCode::Right.into(),
                    },
                    Action::Move,
                )
                .insert(DualAxis::right_stick(), Action::Look)
                .insert(GamepadButtonType::RightTrigger2, Action::ShootPrimary)
                .insert(MouseButton::Left, Action::ShootPrimary)
                .insert(GamepadButtonType::RightTrigger, Action::ShootSecondary)
                .insert(MouseButton::Right, Action::ShootSecondary)
                .insert(GamepadButtonType::South, Action::Jump)
                .insert(KeyCode::Space, Action::Jump)
                .build(),
            can_pick_up_items: CanPickUpItems {},
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        static POST_SIMULATION: &str = "post_simulation";
        app.add_plugin(InputManagerPlugin::<Action>::default())
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
    mut player_character: Query<(
        With<Player>,
        &mut character::Input,
        &ActionState<Action>,
    )>,
) {
    use character::JumpState::*;
    if let Some((_, mut character_input, action_state)) =
        player_character.iter_mut().next()
    {
        // Movement
        if action_state.pressed(Action::Move) {
            let axis_pair =
                action_state.clamped_axis_pair(Action::Move).unwrap();
            character_input.movement_direction =
                Vec3::new(-axis_pair.x(), 0.0, axis_pair.y());
        } else {
            character_input.movement_direction = Vec3::ZERO;
        }

        // Attack
        if action_state.just_pressed(Action::ShootPrimary) {
            character_input.attack = Some(character::AttackState::Primary);
        } else if action_state.just_pressed(Action::ShootSecondary) {
            character_input.attack = Some(character::AttackState::Secondary);
        } else {
            character_input.attack = None;
        }

        // Jump
        character_input.jump = if action_state.just_released(Action::Jump) {
            Some(JumpPressed)
        } else if action_state.pressed(Action::Jump) {
            Some(Charging)
        } else {
            None
        };
    }
}

fn player_looking_input(
    windows: Res<Windows>,
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
        &ActionState<Action>,
    )>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    if let Some((_, player_transform, mut player_input, action_state)) =
        player_character.iter_mut().next()
    {
        if let Some((camera, camera_transform, ..)) = q_camera.iter().next() {
            // directional
            {
                // get the window that the camera is displaying to (or the
                // primary window)
                let wnd =
                    if let bevy::render::camera::RenderTarget::Window(id) =
                        camera.target
                    {
                        windows.get(id).unwrap()
                    } else {
                        windows.get_primary().unwrap()
                    };

                let mouse_moved = motion_evr.iter().next().is_some();

                // check if the cursor is inside the window and get its position
                let look_direction_mouse = if mouse_moved && let Some(cursor_pos_screen_pixels) = wnd.cursor_position()
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
                    let player_plane_point = player_transform.translation();

                    let d = ray_direction.dot(player_plane_normal);
                    // if this is false, line is probably parallel to th
                    // plane.
                    if d.abs() > 0.0001 {
                        let diff =
                            cursor_world_pos_near - player_plane_point;
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
                // Look
                let look_direction_gamepad = if action_state
                    .pressed(Action::Look)
                {
                    let axis_pair =
                        action_state.clamped_axis_pair(Action::Look).unwrap();
                    let dir = Vec3::new(-axis_pair.x(), 0.0, axis_pair.y());
                    if dir.length() > 0.6 {
                        Some(dir)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(look_direction) = look_direction_mouse
                    .or(look_direction_gamepad)
                    .filter(|v| *v != Vec3::ZERO)
                {
                    player_input.looking_direction = look_direction;
                }
            }
        }
    }
}
