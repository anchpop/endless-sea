use bevy::{prelude::*, time::Stopwatch};
use bevy_inspector_egui::Inspectable;
use bevy_rapier3d::prelude::*;

// Plugin
// ======

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(force_movement).add_system(impluse_movement);
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

// Bundle
// ======

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct PlayerCharacter;

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct CharacterMovementProperties {
    pub stopped_friction: f32,
    pub acceleration: f32,
    pub damping_factor: f32,
    pub max_speed: f32,
    pub jump_impulse: f32,
}

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub enum JumpState {
    #[default]
    Normal,
    Charging(Stopwatch),
    JumpPressed(Stopwatch),
}

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct CharacterInput {
    pub direction: Vec3,
    pub jump: JumpState,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_mass_properties: ColliderMassProperties,
    pub restitution: Restitution,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub character_movement_properties: CharacterMovementProperties,
    pub character_input: CharacterInput,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub friction: Friction,
}

impl Default for CharacterBundle {
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
            character_movement_properties: CharacterMovementProperties {
                stopped_friction: 4.0,
                acceleration: 20.0,
                damping_factor: 60.0,
                max_speed: 10.0,
                jump_impulse: 6.0,
            },
            character_input: CharacterInput::default(),
            external_force: ExternalForce::default(),
            external_impulse: ExternalImpulse::default(),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Max,
            },
        }
    }
}
