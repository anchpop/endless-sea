use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Inspectable, Component, Clone, Default)]
pub struct Item;

// Bundle
// ======

#[derive(Bundle)]
pub struct Bundle {
    pub item: Item,
    pub collider: Collider,
    pub sensor: Sensor,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            item: Default::default(),
            collider: Collider::cuboid(0.3, 0.3, 0.3),
            sensor: Sensor::default(),
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {}
}
