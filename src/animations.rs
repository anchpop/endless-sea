use bevy::prelude::*;

use crate::{
    asset_holder,
    character::{self, AnimationState},
    npc, player,
};

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
    assets: Res<asset_holder::AssetHolder>,
    characters: Query<
        (&AnimationState, &Children, With<character::Character>),
        Or<(Changed<AnimationState>, Added<AnimationState>)>,
    >,
    children: Query<(&Children, Without<character::Character>)>,
    mut animations: Query<&mut AnimationPlayer>,
) {
    for (animation_state, character_children, _) in characters.iter() {
        for child in character_children
            .iter()
            .filter_map(|child| children.get(*child).ok())
            .flat_map(|(children, _)| children.iter())
        {
            if let Ok(mut animation_player) = animations.get_mut(*child) {
                let animation = match animation_state {
                    AnimationState::Idle => assets.character_idle.clone(),
                    AnimationState::Walk => assets.character_run.clone(),
                };
                animation_player.play(animation).repeat();
            }
        }
    }
}

fn add_player_model(
    mut commands: Commands,
    assets: Res<asset_holder::AssetHolder>,
    players: Query<Entity, Added<player::Player>>,
) {
    for player in players.iter() {
        commands.entity(player).insert(assets.character.clone());
    }
}

fn add_npc_model(
    mut commands: Commands,
    assets: Res<asset_holder::AssetHolder>,
    npcs: Query<Entity, Added<npc::Npc>>,
) {
    for npc in npcs.iter() {
        commands.entity(npc).insert(assets.character.clone());
    }
}
