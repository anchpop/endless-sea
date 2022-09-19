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
        app.add_system(death).add_system(set_external_impulse);

        if cfg!(debug_assertions) {
            app.register_inspectable::<Health>()
                .register_inspectable::<KnockbackImpulse>();
        }
    }
}

fn death(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut objects: Query<(Entity, &Health, &GlobalTransform)>,
) {
    for (entity, health, transform) in objects.iter_mut() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();

            for i in 0..4 {
                commands
                    .spawn()
                    .insert(RigidBody::Dynamic)
                    .insert(Collider::cuboid(0.1, 0.1, 0.1))
                    .insert(Friction::coefficient(10.0))
                    .insert_bundle(SceneBundle {
                        scene: asset_server
                            .load(&format!("cube/cube.gltf#Scene{i}")),
                        transform: transform.compute_transform(),
                        ..default()
                    })
                    .insert(Name::new("Gib"));
            }
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
