use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

// Bundle
// ======

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Npc {
    pub peaceful: bool,
}

// Components
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {}
}
