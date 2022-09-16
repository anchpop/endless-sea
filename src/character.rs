use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;

// Bundle
// ======

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Player;

#[derive(Inspectable, Reflect, Component, Clone)]
#[reflect(Component)]
pub struct MovementProperties {
    pub stopped_friction: f32,
    pub acceleration: f32,
    pub air_acceleration: f32,
    pub damping_factor: f32,
    pub max_speed: f32,

    pub max_charge_time: Duration,
    pub min_jump_impulse: f32,
    pub max_jump_impulse: f32,
}

impl Default for MovementProperties {
    fn default() -> Self {
        Self {
            stopped_friction: 4.0,
            acceleration: 20.0,
            air_acceleration: 10.0,
            damping_factor: 60.0,
            max_speed: 10.0,

            max_charge_time: Duration::from_secs_f32(0.75),
            min_jump_impulse: 3.0,
            max_jump_impulse: 6.0,
        }
    }
}

#[derive(Component, Clone)]
pub enum JumpState {
    Charging(Stopwatch),
    JumpPressed(Stopwatch),
}

#[derive(Clone)]
pub enum AttackState {
    Primary,
    Secondary,
}

#[derive(Component, Reflect, Inspectable, Clone)]
#[reflect(Component)]
pub struct Character {
    pub on_ground: bool,
    pub max_health: f64,
    pub current_health: f64,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            on_ground: false,
            max_health: 1.0,
            current_health: 1.0,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct Input {
    pub movement_direction: Vec3,
    pub looking_direction: Vec3,
    pub attack: Option<AttackState>,
    pub jump: Option<JumpState>,
}

#[derive(Bundle)]
pub struct Bundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_mass_properties: ColliderMassProperties,
    pub restitution: Restitution,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub character: Character,
    pub movement_properties: MovementProperties,
    pub input: Input,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub friction: Friction,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                0.5,
            ),
            collider_mass_properties: ColliderMassProperties::Mass(1.0),
            restitution: Restitution::coefficient(0.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            character: Character::default(),
            movement_properties: default(),
            input: Input::default(),
            external_force: ExternalForce::default(),
            external_impulse: ExternalImpulse::default(),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Max,
            },
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(force_movement)
            .add_system(impulse_movement)
            .add_system(
                update_grounded
                    .before(impulse_movement)
                    .before(force_movement),
            )
            .add_system(death)
            .register_inspectable::<Character>()
            .register_inspectable::<MovementProperties>();
    }
}

fn force_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &MovementProperties,
        &Velocity,
        &mut ExternalForce,
        &mut Friction,
    )>,
) {
    fn project_onto_plane(v: Vec3, n: Vec3) -> Vec3 {
        v - v.project_onto(n)
    }
    for (
        character,
        input,
        movement_properties,
        velocity,
        mut external_force,
        mut friction,
    ) in characters.iter_mut()
    {
        let input_direction =
            project_onto_plane(input.movement_direction, Vec3::Y);
        let velocity_direction_difference = velocity
            .linvel
            .try_normalize()
            .map(|v| input_direction - project_onto_plane(v, Vec3::Y))
            .unwrap_or(Vec3::ZERO);

        if input_direction != Vec3::ZERO {
            let under_max_speed =
                velocity.linvel.project_onto(input_direction).length()
                    < movement_properties.max_speed;
            let directional_force = if under_max_speed {
                let acceleration = if character.on_ground {
                    movement_properties.acceleration
                } else {
                    movement_properties.air_acceleration
                };
                input_direction * acceleration
            } else {
                Vec3::ZERO
            };
            let damping_force = if character.on_ground {
                velocity_direction_difference
                    * movement_properties.damping_factor
            } else {
                Vec3::ZERO
            };
            external_force.force = directional_force + damping_force;
            friction.coefficient = 0.0;
        } else {
            friction.coefficient = movement_properties.stopped_friction;
            external_force.force = input_direction;
        }
    }
}

fn death(mut commands: Commands, mut characters: Query<(Entity, &Character)>) {
    for (entity, character) in characters.iter_mut() {
        if character.current_health <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn impulse_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &MovementProperties,
        &mut ExternalImpulse,
    )>,
) {
    for (character, input, movement_properties, mut external_impulse) in
        characters.iter_mut()
    {
        if character.on_ground && let Some(JumpState::JumpPressed(watch)) = input.jump.clone()
        {
            let max_charge_time = movement_properties.max_charge_time.as_secs_f32();
            let jump_intensity = watch.elapsed_secs().min(max_charge_time) / max_charge_time;
            let jump_impulse = movement_properties.min_jump_impulse + jump_intensity * (movement_properties.max_jump_impulse - movement_properties.min_jump_impulse);
            external_impulse.impulse = Vec3::new(
                0.,
                jump_impulse,
                0.,
            );
        } else {
            external_impulse.impulse = Vec3::ZERO;
        }
    }
}

fn update_grounded(
    rapier_context: Res<RapierContext>,
    mut player_character: Query<(
        Entity,
        &mut Character,
        &Transform,
        &Collider,
    )>,
) {
    for (entity, mut character, transform, collider) in
        player_character.iter_mut()
    {
        if let Some((_entity, _toi)) = rapier_context.cast_shape(
            // TODO: This is a hack to make sure the ray doesn't start inside
            // the ground if the collider is slightly underground,
            // bus will cause rare false positives when the player'
            // s head hits the ceiling.
            transform.translation + Vec3::Y * 0.05,
            transform.rotation,
            Vec3::NEG_Y,
            collider,
            0.2,
            QueryFilter {
                exclude_collider: Some(entity),
                ..default()
            },
        ) {
            character.on_ground = true;
        } else {
            character.on_ground = false;
        }
    }
}
