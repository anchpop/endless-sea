use std::collections::HashSet;

use bevy::{prelude::*, time::Stopwatch};
use bevy_mod_wanderlust::{
    ControllerBundle, ControllerInput, ControllerPhysicsBundle,
    ControllerSettings,
};
use bevy_rapier3d::prelude::*;

use crate::{
    helpers::{self, *},
    item, object,
    reticle::Reticle,
};

// Bundle
// ======

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
pub struct MovementProperties {
    pub stopped_friction: f32,
    pub acceleration: f32,
    pub air_acceleration: f32,
    pub damping_factor: f32,
    pub max_speed: f32,
}

impl Default for MovementProperties {
    fn default() -> Self {
        Self {
            stopped_friction: 4.0,
            acceleration: 20.0,
            air_acceleration: 10.0,
            damping_factor: 60.0,
            max_speed: 10.0,
        }
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub enum AttackState {
    Primary,
    Secondary,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Character {
    pub on_ground: bool,
}

#[derive(Component, Default, Clone)]
pub struct Input {
    pub movement_direction: Vec3,
    pub looking_direction: Vec3,
    pub attack: Option<AttackState>,
    pub jump: bool,
    pub switch_hands: bool,
}

#[derive(Component, Default, Clone)]
pub struct WalkForce(pub Vec3);

#[derive(Component, Clone, Default, Debug)]
pub struct Inventory {
    pub hand: Option<item::HeldItem>,
    pub belt: Option<item::HeldItem>,
    pub backpack: Vec<item::HeldItem>,
}

#[derive(Component, Clone, Default)]
pub struct CanPickUpItems {}

#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AnimationState {
    Idle,
    Walk,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

// Bundle
// ======

#[derive(Bundle)]
pub struct Bundle {
    // physics
    pub character_controller: ControllerBundle,

    // character stuff
    pub character: Character,
    pub movement_properties: MovementProperties,
    pub input: Input,
    pub walk_force: WalkForce,
    pub inventory: Inventory,
    pub knockback_impulse: object::KnockbackImpulse,
    pub health: object::Health,

    // animation
    pub animation_state: AnimationState,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            character_controller: ControllerBundle {
                settings: ControllerSettings {
                    force_scale: Vec3::new(1.0, 0.0, 1.0),
                    ..ControllerSettings::character()
                },
                physics: ControllerPhysicsBundle {
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Max,
                    },
                    collider: Collider::capsule(
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::new(0.0, 1.5, 0.0),
                        0.4,
                    ),
                    locked_axes: LockedAxes::ROTATION_LOCKED,
                    ..Default::default()
                },
                ..Default::default()
            },
            character: Character::default(),
            movement_properties: default(),
            input: Input::default(),
            walk_force: WalkForce::default(),
            inventory: Inventory::default(),
            knockback_impulse: object::KnockbackImpulse::default(),
            health: object::Health::default(),
            animation_state: AnimationState::default(),
        }
    }
}

impl Bundle {
    pub fn from_transform(transform: Transform) -> Self {
        let default = Self::default();
        Self {
            character_controller: ControllerBundle {
                transform,
                ..default.character_controller
            },
            ..default
        }
    }
}

// Plugin
// ======

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum MovementSet {
    CollectInfo,
    ComputeForce,
    ApplyForces,
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_grounded.in_set(MovementSet::CollectInfo),
                force_movement.in_set(MovementSet::ComputeForce),
                move_character_controller.in_set(MovementSet::ApplyForces),
                attack,
                rotate_character,
                check_no_character_and_object,
                pick_up_items,
                control_reticle_based_on_inventory,
                switch_hands,
                increment_cooldown_timers,
                update_character_animation_state,
            ),
        );

        if cfg!(debug_assertions) {
            app.register_type::<Character>()
                .register_type::<MovementProperties>();
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
    mut characters: Query<(
        Entity,
        &Input,
        &GlobalTransform,
        &Transform,
        &mut Inventory,
    )>,
    mut character_q: Query<(
        &mut object::Health,
        &mut object::KnockbackImpulse,
    )>,
) {
    for (entity, input, transform, transform_local, mut inventory) in
        characters.iter_mut()
    {
        if let Some(held_item) = &mut inventory.hand && let Some(attack) = &input.attack {
            let item = &held_item.item;
            let can_use = match &held_item.time_since_last_use {
                Some(time) => time.elapsed() > item.cooldown(),
                None => true,
            };
            if can_use {
                held_item.time_since_last_use = Some(Stopwatch::new());

                match (item, attack) {
                    (item::Item::Gun, AttackState::Primary) => {
                        if let Some((entity, _toi)) = rapier_context.cast_ray(
                            transform.translation(),
                            input.looking_direction.normalize_or_zero(),
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
                                impulse.0 =
                                    input.looking_direction.normalize_or_zero() * 10.0;
                            }
                        }
                    }
                    (item::Item::Gun, AttackState::Secondary) => {}
                    (item::Item::Sword, AttackState::Primary) => {
                        let attack_distance = 2.5;
                        let attempts = (31, 3);
                        let angle = (0.25 * std::f32::consts::PI, 0.1 * std::f32::consts::PI);
                        let hit_entities =
                            (0..attempts.0)
                            .flat_map(|attempt_x| (0..attempts.1).map(
                                move |attempt_y| (attempt_x, attempt_y))
                            )
                            .filter_map(|attempt| {
                                // GlobalTransform::up() gives the global up transform, so we use the local transform to get the local up vector.
                                let ray_dir = Quat::from_axis_angle(transform_local.up(), helpers::lerp(
                                    -angle.0,
                                    angle.0,
                                    attempt.0 as f32 / (attempts.0 - 1) as f32,
                                )) * Quat::from_axis_angle(transform_local.right(), helpers::lerp(
                                    -angle.1,
                                    angle.1,
                                    attempt.1 as f32 / (attempts.1 - 1) as f32,
                                )) * input.looking_direction.normalize_or_zero();
                                rapier_context.cast_ray(
                                    transform.translation(),
                                    ray_dir,
                                    attack_distance,
                                    true,
                                    QueryFilter {
                                        exclude_collider: Some(entity),
                                        ..default()
                                    },
                                )
                            })
                            .map(|(entity, _toi)| entity)
                            .collect::<HashSet<_>>();
                        for entity in hit_entities {
                            if let Ok((mut health, mut impulse)) =
                                character_q.get_mut(entity)
                            {
                                health.current -= 0.5;
                                impulse.0 =
                                    input.looking_direction.normalize_or_zero() * 10.0;
                            }
                        }
                    }
                    (item::Item::Sword, AttackState::Secondary) => {}
                }
            }

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

fn move_character_controller(
    mut characters: Query<
        (&mut ControllerInput, &mut WalkForce, &Input),
        (With<Character>, Without<object::Object>),
    >,
) {
    for (mut controller_input, mut walk_force, input) in characters.iter_mut() {
        let force = walk_force.0;
        controller_input.movement = force;
        controller_input.jumping = input.jump;
        walk_force.0 = Vec3::ZERO;
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

fn pick_up_items(
    mut commands: Commands,
    mut characters: Query<(&Transform, &mut Inventory), With<CanPickUpItems>>,
    item: Query<&item::Item>,
    rapier_context: Res<RapierContext>,
) {
    for (transform, mut inventory) in characters.iter_mut() {
        rapier_context.intersections_with_shape(
            transform.translation,
            Quat::IDENTITY,
            &Collider::ball(1.0),
            QueryFilter::default(),
            |entity| {
                let Ok(item) = item.get(entity) else {
                    return true;
                };

                match *inventory {
                    Inventory { hand: None, .. } => {
                        inventory.hand =
                            Some(item::HeldItem::new(item.clone()));
                    }
                    Inventory {
                        hand: Some(_),
                        belt: None,
                        ..
                    } => {
                        inventory.belt =
                            Some(item::HeldItem::new(item.clone()));
                    }
                    Inventory {
                        hand: Some(_),
                        belt: Some(_),
                        backpack: _,
                    } => inventory
                        .backpack
                        .push(item::HeldItem::new(item.clone())),
                }
                commands.entity(entity).despawn_recursive();
                false
            },
        );
    }
}

fn check_no_character_and_object(
    characters_with_objects: Query<(With<Character>, With<object::Object>)>,
) {
    for _ in &characters_with_objects {
        panic!("Character and Object components cannot be on the same entity");
    }
}

fn control_reticle_based_on_inventory(
    mut reticles: Query<(&Inventory, &mut Reticle)>,
) {
    for (inventory, mut reticle) in reticles.iter_mut() {
        match inventory {
            Inventory {
                hand:
                    Some(item::HeldItem {
                        item: item::Item::Gun,
                        ..
                    }),
                ..
            } => {
                reticle.enabled = true;
            }
            _ => {
                reticle.enabled = false;
            }
        }
    }
}

fn switch_hands(mut characters: Query<(&Input, &mut Inventory)>) {
    for (input, mut inventory) in characters.iter_mut() {
        if input.switch_hands {
            let left = inventory.belt.clone();
            let right = inventory.hand.clone();
            inventory.belt = right;
            inventory.hand = left;
        }
    }
}

fn increment_cooldown_timers(
    time: Res<Time>,
    mut inventories: Query<(&mut Inventory,)>,
) {
    for (mut inventory,) in inventories.iter_mut() {
        if let Some(time_since) = &mut inventory
            .hand
            .as_mut()
            .and_then(|item| item.time_since_last_use.as_mut())
        {
            time_since.tick(time.delta());
        }
        if let Some(time_since) = &mut inventory
            .belt
            .as_mut()
            .and_then(|item| item.time_since_last_use.as_mut())
        {
            time_since.tick(time.delta());
        }
        for held_item in inventory.backpack.iter_mut() {
            if let Some(time_since_last_use) =
                &mut held_item.time_since_last_use
            {
                time_since_last_use.tick(time.delta());
            }
        }
    }
}

fn update_character_animation_state(
    mut characters: Query<(&Input, &mut AnimationState)>,
) {
    for (input, mut animation_state) in characters.iter_mut() {
        let mut upsert = |new_state: AnimationState| {
            if animation_state.as_ref() != &new_state {
                *animation_state = new_state;
            }
        };
        if input.movement_direction.length() > 0.1 {
            upsert(AnimationState::Walk);
        } else {
            upsert(AnimationState::Idle);
        }
    }
}
