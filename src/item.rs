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
    pub rigid_body: RigidBody,
    pub collider: Collider,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            item: Default::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(0.3, 0.3, 0.3),
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {}
}
