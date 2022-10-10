use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Inspectable, Component, Clone)]
pub enum Item {
    Sword,
}

impl ToString for Item {
    fn to_string(&self) -> String {
        match self {
            Item::Sword => "Sword".to_string(),
        }
    }
}

// Bundle
// ======

#[derive(Bundle)]
pub struct Bundle {
    pub item: Item,
    pub collider: Collider,
    pub sensor: Sensor,
}

impl Bundle {
    pub fn sword() -> Self {
        Self {
            item: Item::Sword,
            collider: Collider::cuboid(0.3, 0.3, 0.3),
            sensor: Sensor::default(),
        }
    }
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {}
}
