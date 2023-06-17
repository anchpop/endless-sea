use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use bevy_rapier3d::prelude::*;

// Components
// ==========

#[derive(Component, Clone, Debug)]
pub(crate) enum Item {
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
    pub(crate) fn cooldown(&self) -> Duration {
        match self {
            Item::Sword => Duration::from_millis(200),
            Item::Gun => Duration::from_millis(1000),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HeldItem {
    pub(crate) item: Item,
    pub(crate) time_since_last_use: Option<Stopwatch>,
}

impl HeldItem {
    pub(crate) fn new(item: Item) -> Self {
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
pub(crate) struct Bundle {
    pub(crate) item: Item,
    pub(crate) collider: Collider,
}

impl Bundle {
    pub(crate) fn sword() -> Self {
        Self {
            item: Item::Sword,
            collider: Collider::cuboid(0.3, 0.3, 0.3),
        }
    }
    pub(crate) fn gun() -> Self {
        Self {
            item: Item::Gun,
            collider: Collider::cuboid(0.3, 0.3, 0.3),
        }
    }
}

// Plugin
// ======

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {}
}
