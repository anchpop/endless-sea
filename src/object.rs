use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Inspectable, Component, Clone, Default)]
pub struct Object;

#[derive(Inspectable, Reflect, Component, Clone)]
#[reflect(Component)]
pub struct Health {
    pub max: f64,
    pub current: f64,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            max: 1.0,
            current: 1.0,
        }
    }
}

#[derive(Inspectable, Component, Default, Clone)]
pub struct KnockbackImpulse(pub Vec3);

#[derive(Bundle, Default)]
pub struct Bundle {
    pub health: Health,
    pub knockback_impulse: KnockbackImpulse,
    pub external_impulse: ExternalImpulse,
    pub external_force: ExternalForce,
    pub object: Object,
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Health>()
            .register_inspectable::<KnockbackImpulse>()
            .add_system(death)
            .add_system(set_external_impulse);
    }
}

fn death(mut commands: Commands, mut objects: Query<(Entity, &Health)>) {
    for (entity, health) in objects.iter_mut() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn set_external_impulse(
    mut characters: Query<(
        &mut ExternalImpulse,
        &mut KnockbackImpulse,
        With<Object>,
    )>,
) {
    for (mut external_impulse, mut knockback_impulse, _) in
        characters.iter_mut()
    {
        external_impulse.impulse = knockback_impulse.0;
        knockback_impulse.0 = Vec3::ZERO;
    }
}
