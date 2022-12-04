use std::{collections::HashSet, time::Duration};

use bevy::{prelude::*, time::Stopwatch};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;

use crate::{
    helpers::{self, *},
    item, object,
    reticle::Reticle,
};

// Bundle
// ======

#[derive(Inspectable, Reflect, Component, Clone)]
#[reflect(Component)]
pub struct MovementProperties {
    pub stopped_friction: f32,
    pub air_acceleration: f32,
    pub max_speed: f32,

    pub max_charge_time: Duration,
    pub min_jump_impulse: f32,
    pub max_jump_impulse: f32,
}

impl Default for MovementProperties {
    fn default() -> Self {
        Self {
            stopped_friction: 8.0,
            air_acceleration: 10.0,
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
    pub ground_velocity: Option<Vec3>,
}

#[derive(Component, Default, Clone)]
pub struct Input {
    pub movement_direction: Vec3,
    pub looking_direction: Vec3,
    pub attack: Option<AttackState>,
    pub jump: Option<JumpState>,
    pub switch_hands: bool,
}

#[derive(Component, Default, Clone)]
pub struct JumpInfo {
    charge: Option<Stopwatch>,
    time_since_last_jump: Option<Stopwatch>,
}

#[derive(Component, Default, Clone)]
pub struct WalkForce(pub Vec3);

#[derive(Component, Default, Clone)]
pub struct WalkImpulse(pub Vec3);

#[derive(Component, Default, Clone)]
pub struct JumpImpulse(pub Vec3);

#[derive(Component, Clone, Default, Debug)]
pub struct Inventory {
    pub hand: Option<item::HeldItem>,
    pub belt: Option<item::HeldItem>,
    pub backpack: Vec<item::HeldItem>,
}

#[derive(Inspectable, Component, Clone, Default)]
pub struct CanPickUpItems {}

#[derive(Inspectable, Component, Clone, PartialEq, Eq, Hash, Debug)]
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
    pub jump_charge: JumpInfo,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
    pub friction: Friction,
    pub walk_force: WalkForce,
    pub walk_impulse: WalkImpulse,
    pub jump_impulse: JumpImpulse,
    pub inventory: Inventory,
    pub knockback_impulse: object::KnockbackImpulse,
    pub health: object::Health,

    // animation
    pub animation_state: AnimationState,
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
            jump_charge: JumpInfo::default(),
            external_force: ExternalForce::default(),
            external_impulse: ExternalImpulse::default(),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            walk_force: WalkForce::default(),
            walk_impulse: WalkImpulse::default(),
            jump_impulse: JumpImpulse::default(),
            inventory: Inventory::default(),
            knockback_impulse: object::KnockbackImpulse::default(),
            health: object::Health::default(),
            animation_state: AnimationState::default(),
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_jump_state)
            .add_system(walk_movement)
            .add_system(jump_movement.before(attack))
            .add_system(
                update_grounded.before(jump_movement).before(walk_movement),
            )
            .add_system(attack)
            .add_system(rotate_character)
            .add_system(check_no_character_and_object)
            .add_system(pick_up_items)
            .add_system(control_reticle_based_on_inventory)
            .add_system(switch_hands)
            .add_system(increment_cooldown_timers)
            .add_system(update_character_animation_state)
            .add_system(set_external_force)
            .add_system(
                set_external_impulse
                    .after(walk_movement)
                    .after(jump_movement),
            );

        if cfg!(debug_assertions) {
            app.register_inspectable::<Character>()
                .register_inspectable::<MovementProperties>();
        }
    }
}

fn update_jump_state(
    mut query: Query<(&Input, &mut JumpInfo)>,
    time: Res<Time>,
) {
    for (input, mut jump_charge) in query.iter_mut() {
        jump_charge
            .time_since_last_jump
            .as_mut()
            .map(|time_since_last_jump| {
                time_since_last_jump.tick(time.delta())
            });
        let charge = &mut jump_charge.charge;
        match input.jump {
            Some(JumpState::Charging) => match charge {
                Some(ref mut charge) => {
                    charge.tick(time.delta());
                }
                None => {
                    *charge = Some(Stopwatch::new());
                    jump_charge.time_since_last_jump = Some(Stopwatch::new());
                }
            },
            Some(JumpState::JumpPressed) => {}
            None => {
                *charge = None;
            }
        }
    }
}

fn walk_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &MovementProperties,
        &Velocity,
        &mut WalkForce,
        &mut WalkImpulse,
    )>,
) {
    for (
        character,
        input,
        movement_properties,
        velocity,
        mut walk_force,
        mut walk_impulse,
    ) in characters.iter_mut()
    {
        dbg!(velocity.linvel.y);
        let input_direction = {
            let projected =
                project_onto_plane(input.movement_direction, Vec3::Y);
            if projected.length() > 1.0 {
                projected.normalize()
            } else {
                projected
            }
        };
        if let Some(ground_velocity) = character.ground_velocity {
            if input_direction != Vec3::ZERO {
                let goal_speed = ground_velocity
                    + input_direction * movement_properties.max_speed;
                let velocity_change =
                    -project_onto_plane(velocity.linvel, Vec3::Y) + goal_speed;
                walk_impulse.0 = velocity_change;
            } else {
                walk_force.0 = (-velocity.linvel + ground_velocity)
                    * movement_properties.stopped_friction;
            }
        } else {
        }
    }
}

fn jump_movement(
    mut characters: Query<(
        &Character,
        &Input,
        &JumpInfo,
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
        if character.ground_velocity.is_some() && let (Some(JumpState::JumpPressed), Some(watch)) = (input.jump.clone(), jump_charge.charge.clone())
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
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut player_character: Query<(
        Entity,
        &mut Character,
        &MovementProperties,
        &Transform,
        &Collider,
    )>,
    velocity: Query<&Velocity>,
) {
    for (entity, mut character, movement_properties, transform, collider) in
        player_character.iter_mut()
    {
        // TODO: This is a hack to make sure the ray doesn't start inside the
        // ground if the collider is slightly underground, but will cause rare
        // false positives when the player's head hits the ceiling.
        let vertical_offset = Vec3::Y * 0.05;

        let jump_dist =
            time.delta_seconds() * movement_properties.min_jump_impulse;
        let vertical_noise = time.delta_seconds() * 8.0;
        let _dist_from_ground_to_check = jump_dist + vertical_noise;

        if let Some((entity, _toi)) = rapier_context.cast_shape(
            transform.translation + vertical_offset,
            transform.rotation,
            Vec3::NEG_Y,
            collider,
            jump_dist + vertical_noise,
            QueryFilter {
                exclude_collider: Some(entity),
                ..default()
            },
        ) {
            println!("Character grounded");
            // Todo: if object is rotating (eg it is a boat), we should also
            // incorporate the object's angular velocity here.
            if let Ok(velocity) = velocity.get(entity) {
                character.ground_velocity = Some(velocity.linvel);
            } else {
                character.ground_velocity = Some(Vec3::ZERO);
            }
        } else {
            println!("Character ungrounded");
            character.ground_velocity = None;
        }
    }
}

// useless rn
fn set_external_force(
    mut characters: Query<
        (&mut ExternalForce, &mut WalkForce),
        (With<Character>, Without<object::Object>),
    >,
) {
    for (mut external_force, mut walk_force) in characters.iter_mut() {
        external_force.force = walk_force.0;
        walk_force.0 = Vec3::ZERO;
        // set external force appropriately
    }
}

fn set_external_impulse(
    mut characters: Query<
        (
            &mut ExternalImpulse,
            &mut JumpImpulse,
            &mut WalkImpulse,
            &mut object::KnockbackImpulse,
        ),
        (With<Character>, Without<object::Object>),
    >,
) {
    for (
        mut external_impulse,
        mut jump_impulse,
        mut knockback_impulse,
        mut walk_impulse,
    ) in characters.iter_mut()
    {
        external_impulse.impulse =
            jump_impulse.0 + knockback_impulse.0 + walk_impulse.0;
        jump_impulse.0 = Vec3::ZERO;
        knockback_impulse.0 = Vec3::ZERO;
        walk_impulse.0 = Vec3::ZERO;
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
    mut characters: Query<(Entity, &mut Inventory, With<CanPickUpItems>)>,
    item: Query<&item::Item>,
    rapier_context: Res<RapierContext>,
) {
    for (character_entity, mut inventory, _) in characters.iter_mut() {
        /* Iterate through all the intersection pairs involving a specific
        collider. */
        for (item_entity, _, intersecting) in
            rapier_context.intersections_with(character_entity)
        {
            if intersecting {
                if let Ok(item) = item.get(item_entity) {
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
                    commands.entity(item_entity).despawn_recursive();
                }
            }
        }
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
