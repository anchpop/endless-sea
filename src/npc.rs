use bevy::prelude::*;

use crate::{character, helpers::project_onto_plane, player};

// Bundle
// ======

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub(crate) struct Npc {
    pub(crate) peaceful: bool,
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
            if project_onto_plane(npc_transform.translation, Vec3::Y).distance(
                project_onto_plane(player_transform.translation, Vec3::Y),
            ) > 2.0
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

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(npc_input);
    }
}
