use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{character, player};

// Bundle
// ======

#[derive(Inspectable, Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Npc {
    pub peaceful: bool,
}

// Components
// ======

fn npc_input(
    mut npcs: Query<(With<Npc>, &mut character::Input, &Transform)>,
    player: Query<(With<player::Player>, &Transform)>,
) {
    if let Some((_, player_transform)) = player.iter().next() {
        for (_, mut npc_input, npc_transform) in npcs.iter_mut() {
            npc_input.looking_direction =
                player_transform.translation - npc_transform.translation;
            if npc_transform
                .translation
                .distance(player_transform.translation)
                > 2.0
            {
                npc_input.movement_direction = (player_transform.translation
                    - npc_transform.translation)
                    .try_normalize()
                    .unwrap_or(Vec3::ZERO);
            } else {
                npc_input.movement_direction = Vec3::ZERO;
            }
        }
    }
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(npc_input);
    }
}
