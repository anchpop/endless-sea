use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Inspectable, Component, Clone, Debug)]
pub enum Item {
    Sword,
    Gun,
}

impl ToString for Item {
    fn to_string(&self) -> String {
        match self {
            Item::Sword => "Sword".to_string(),
            Item::Gun => "Gun".to_string(),
        }
    }
}

impl From<&Item> for String {
    fn from(i: &Item) -> String {
        i.to_string()
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
    pub fn gun() -> Self {
        Self {
            item: Item::Gun,
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