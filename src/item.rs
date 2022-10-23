use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
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

impl Item {
    pub fn cooldown(&self) -> Duration {
        match self {
            Item::Sword => Duration::from_millis(200),
            Item::Gun => Duration::from_millis(1000),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HeldItem {
    pub item: Item,
    pub time_since_last_use: Option<Stopwatch>,
}

impl HeldItem {
    pub fn new(item: Item) -> Self {
        Self {
            item,
            time_since_last_use: None,
        }
    }
}

impl ToString for HeldItem {
    fn to_string(&self) -> String {
        let cooldown_timer =
            if let Some(time_since_last_use) = &self.time_since_last_use {
                if time_since_last_use.elapsed() < self.item.cooldown() {
                    format!(
                        " ({:.2}s)",
                        (self.item.cooldown() - time_since_last_use.elapsed())
                            .as_secs_f32()
                    )
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };
        format!("{}{}", self.item.to_string(), cooldown_timer)
    }
}

impl From<&HeldItem> for String {
    fn from(i: &HeldItem) -> String {
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
