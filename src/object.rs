use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Component, Clone, Default)]
pub struct Lifetime {
    pub time: Timer,
    pub shrink_away: bool,
}

#[derive(Component, Clone, Default)]
pub struct Object;

#[derive(Reflect, Component, Clone, Debug)]
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

#[derive(Component, Default, Clone)]
pub struct ExplodeIntoPieces {
    pub pieces: Vec<(Handle<Scene>, Collider)>,
    pub shrink_away: bool,
}

#[derive(Component, Default, Clone)]
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
        app.add_systems(
            Update,
            (
                death,
                set_external_impulse,
                count_down_lifetime,
                shrink_away,
            ),
        );
    }
}

fn death(
    mut commands: Commands,
    mut objects: Query<(
        Entity,
        &Health,
        &GlobalTransform,
        Option<&ExplodeIntoPieces>,
    )>,
) {
    for (entity, health, transform, pieces) in objects.iter_mut() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();

            if let Some(ExplodeIntoPieces {
                pieces,
                shrink_away,
            }) = pieces
            {
                for (scene, collider) in pieces.iter().cloned() {
                    commands.spawn((
                        SceneBundle {
                            scene,
                            transform: transform.compute_transform(),
                            ..default()
                        },
                        RigidBody::Dynamic,
                        collider,
                        Dominance::group(-1),
                        Friction::coefficient(10.0),
                        Name::new("Gib"),
                        Lifetime {
                            time: Timer::from_seconds(10.0, TimerMode::Once),
                            shrink_away: *shrink_away,
                        },
                    ));
                }
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

fn count_down_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut objects: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in objects.iter_mut() {
        lifetime.time.tick(time.delta());
        if lifetime.time.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn shrink_away(mut objects: Query<(&Lifetime, &mut Transform)>) {
    for (lifetime, mut transform) in objects.iter_mut() {
        if lifetime.shrink_away {
            let max = lifetime.time.duration().as_millis() as f32;
            let time_remaining =
                max - lifetime.time.elapsed().as_millis() as f32;
            if time_remaining > 0.0 {
                let scale = (time_remaining / max).powf(0.5);
                transform.scale = Vec3::splat(scale);
            }
        }
    }
}
