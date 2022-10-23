use bevy::prelude::*;

use crate::{assets, character, npc, player};

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate)
            .add_system(add_player_model)
            .add_system(add_npc_model);
    }
}

fn animate(
    assets: Res<assets::AssetHolder>,
    characters: Query<(&Children, With<character::Character>)>,
    children: Query<(&Children, Without<character::Character>)>,
    mut animations: Query<&mut AnimationPlayer>,
    mut playing: Local<bool>,
) {
    if !*playing {
        *playing = true;
        for child in characters
            .iter()
            .flat_map(|(character_children, _)| character_children.iter())
            .filter_map(|child| children.get(*child).ok())
            .flat_map(|(children, _)| children.iter())
        {
            if let Ok(mut animation_player) = animations.get_mut(*child) {
                animation_player.play(assets.character_run.clone()).repeat();
            }
        }
    }
}

fn add_player_model(
    mut commands: Commands,
    assets: Res<assets::AssetHolder>,
    players: Query<(Entity, Added<player::Player>)>,
) {
    for (player, added) in players.iter() {
        if added {
            commands.entity(player).insert(assets.character.clone());
        }
    }
}

fn add_npc_model(
    mut commands: Commands,
    assets: Res<assets::AssetHolder>,
    npcs: Query<(Entity, Added<npc::Npc>)>,
) {
    for (npc, added) in npcs.iter() {
        if added {
            commands.entity(npc).insert(assets.character.clone());
        }
    }
}
