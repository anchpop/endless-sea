use bevy::prelude::*;

use crate::{assets, character};

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate);
    }
}

fn animate(
    assets: Res<assets::Assets>,
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
