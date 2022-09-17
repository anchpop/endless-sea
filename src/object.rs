use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

// Components
// ==========

#[derive(Inspectable, Reflect, Component, Clone)]
#[reflect(Component)]
pub struct Health {
    pub max_health: f64,
    pub current_health: f64,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            max_health: 1.0,
            current_health: 1.0,
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Health>().add_system(death);
    }
}

fn death(mut commands: Commands, mut objects: Query<(Entity, &Health)>) {
    for (entity, health) in objects.iter_mut() {
        if health.current_health <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
