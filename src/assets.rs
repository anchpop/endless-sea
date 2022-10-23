use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Clone)]
pub struct Assets {
    #[asset(path = "floor/floor.glb#Scene0")]
    pub floor: Handle<Scene>,

    #[asset(path = "character/casual_male.glb#Scene0")]
    pub character: Handle<Scene>,
    #[asset(path = "character/casual_male.glb#Animation8")]
    pub character_run: Handle<AnimationClip>,

    #[asset(path = "cube/cube.glb#Scene0")]
    pub cube: Handle<Scene>,
    #[asset(path = "sword/sword.glb#Scene0")]
    pub sword: Handle<Scene>,
    #[asset(path = "gun/gun.glb#Scene0")]
    pub gun: Handle<Scene>,
}
