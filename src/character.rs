use crate::helpers::*;
use crate::object;

use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;

// Bundle
// ======

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
    Charging,
    JumpPressed,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub enum AttackState {
    Primary,
    Secondary,
}

#[derive(Component, Reflect, Inspectable, Default, Clone)]
#[reflect(Component)]
pub struct Character {
    pub on_ground: bool,
}

#[derive(Component, Default, Clone)]
pub struct Input {
    pub movement_direction: Vec3,
    pub looking_direction: Vec3,
    pub attack: Option<AttackState>,
    pub jump: Option<JumpState>,
}

#[derive(Component, Default, Clone)]
pub struct JumpCharge {
    charge: Option<Stopwatch>,
}

#[derive(Component, Default, Clone)]
pub struct WalkForce(pub Vec3);

#[derive(Component, Default, Clone)]
pub struct JumpImpulse(pub Vec3);

#[derive(Bundle)]
pub struct Bundle {
    // physics
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collider_mass_properties: ColliderMassProperties,
    pub restitution: Restitution,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,

    // character stuff
    pub character: Character,
    pub movement_properties: MovementProperties,
    pub input: Input,
    pub jump_charge: JumpCharge,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub friction: Friction,
    pub walk_force: WalkForce,
    pub jump_impulse: JumpImpulse,
    pub knockback_impulse: object::KnockbackImpulse,
    pub health: object::Health,
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
            jump_charge: JumpCharge::default(),
            external_force: ExternalForce::default(),
            external_impulse: ExternalImpulse::default(),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            walk_force: WalkForce::default(),
            jump_impulse: JumpImpulse::default(),
            knockback_impulse: object::KnockbackImpulse::default(),
            health: object::Health::default(),
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_jump_state)
            .add_system(force_movement)
            .add_system(impulse_movement.before(attack))
            .add_system(
                update_grounded
                    .before(impulse_movement)
                    .before(force_movement),
            )
            .add_system(attack)
            .add_system(set_external_force)
            .add_system(set_external_impulse)
            .add_system(rotate_character)
            .add_system(check_no_character_and_object);

        if cfg!(debug_assertions) {
            app.register_inspectable::<Character>()
                .register_inspectable::<MovementProperties>();
        }
    }
}

fn update_jump_state(
    mut query: Query<(&Input, &mut JumpCharge)>,
    time: Res<Time>,
) {
    for (input, mut jump_charge) in query.iter_mut() {
        let charge = &mut jump_charge.charge;
        match input.jump {
            Some(JumpState::Charging) => match charge {
                Some(ref mut charge) => {
                    charge.tick(time.delta());
                }
                None => {
                    *charge = Some(Stopwatch::new());
                }
            },
            Some(JumpState::JumpPressed) => {}
            None => {
                *charge = None;
            }
        }
    }
}

fn force_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &MovementProperties,
        &Velocity,
        &mut WalkForce,
        &mut Friction,
    )>,
) {
    for (
        character,
        input,
        movement_properties,
        velocity,
        mut walk_force,
        mut friction,
    ) in characters.iter_mut()
    {
        let input_direction = {
            let projected =
                project_onto_plane(input.movement_direction, Vec3::Y);
            if projected.length() > 1.0 {
                projected.normalize()
            } else {
                projected
            }
        };
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
            walk_force.0 = directional_force + damping_force;
            friction.coefficient = 0.0;
        } else {
            friction.coefficient = movement_properties.stopped_friction;
            walk_force.0 = input_direction;
        }
    }
}

fn attack(
    rapier_context: Res<RapierContext>,
    mut characters: Query<(Entity, &Input, &GlobalTransform)>,
    mut character_q: Query<(
        &mut object::Health,
        &mut object::KnockbackImpulse,
    )>,
) {
    for (entity, input, transform) in characters.iter_mut() {
        if let Some(AttackState::Primary) = input.attack {
            if let Some((entity, _toi)) = rapier_context.cast_ray(
                transform.translation(),
                input.looking_direction,
                1000.0,
                true,
                QueryFilter {
                    exclude_collider: Some(entity),
                    ..default()
                },
            ) {
                if let Ok((mut health, mut impulse)) =
                    character_q.get_mut(entity)
                {
                    health.current -= 0.5;
                    impulse.0 = input
                        .looking_direction
                        .try_normalize()
                        .unwrap_or(Vec3::ZERO)
                        * 10.0;
                }
            }
        }
    }
}

fn impulse_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &JumpCharge,
        &MovementProperties,
        &mut JumpImpulse,
    )>,
) {
    for (
        character,
        input,
        jump_charge,
        movement_properties,
        mut external_impulse,
    ) in characters.iter_mut()
    {
        if character.on_ground && let (Some(JumpState::JumpPressed), Some(watch)) = (input.jump.clone(), jump_charge.charge.clone())
        {
            let max_charge_time = movement_properties.max_charge_time.as_secs_f32();
            let jump_intensity = watch.elapsed_secs().min(max_charge_time) / max_charge_time;
            let jump_impulse = movement_properties.min_jump_impulse + jump_intensity * (movement_properties.max_jump_impulse - movement_properties.min_jump_impulse);
            external_impulse.0 = Vec3::new(
                0.,
                jump_impulse,
                0.,
            );
        } else {
            external_impulse.0 = Vec3::ZERO;
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

fn set_external_force(
    mut characters: Query<(
        &mut ExternalForce,
        &mut WalkForce,
        With<Character>,
        Without<object::Object>,
    )>,
) {
    for (mut external_force, mut walk_force, _, _) in characters.iter_mut() {
        external_force.force = walk_force.0;
        walk_force.0 = Vec3::ZERO;
    }
}

fn set_external_impulse(
    mut characters: Query<(
        &mut ExternalImpulse,
        &mut JumpImpulse,
        &mut object::KnockbackImpulse,
        With<Character>,
        Without<object::Object>,
    )>,
) {
    for (mut external_impulse, mut jump_impulse, mut knockback_impulse, _, _) in
        characters.iter_mut()
    {
        external_impulse.impulse = jump_impulse.0 + knockback_impulse.0;
        jump_impulse.0 = Vec3::ZERO;
        knockback_impulse.0 = Vec3::ZERO;
    }
}

fn rotate_character(mut characters: Query<(&mut Transform, &Input)>) {
    for (mut transform, input) in characters.iter_mut() {
        let looking_direction =
            project_onto_plane(input.looking_direction, Vec3::Y);
        if looking_direction != Vec3::ZERO {
            let up = transform.up();
            let translation = transform.translation;
            transform.look_at(translation + looking_direction, up);
        }
    }
}

fn check_no_character_and_object(
    characters_with_objects: Query<(With<Character>, With<object::Object>)>,
) {
    for _ in characters_with_objects.iter() {
        panic!("Character and Object components cannot be on the same entity");
    }
}
