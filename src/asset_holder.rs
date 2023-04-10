use bevy::prelude::*;

#[derive(Resource)]
pub struct AssetHolder {
    pub floor: Handle<Scene>,

    pub character: Handle<Scene>,
    pub character_run: Handle<AnimationClip>,
    pub character_idle: Handle<AnimationClip>,

    pub cube: Handle<Scene>,
    pub sword: Handle<Scene>,
    pub gun: Handle<Scene>,
}

pub fn load_assets(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let assets = AssetHolder {
        floor: asset_server.load("floor/floor.glb#Scene0"),
        character: asset_server.load("character/casual_male.glb#Scene0"),
        character_run: asset_server
            .load("character/casual_male.glb#Animation9"),
        character_idle: asset_server
            .load("character/casual_male.glb#Animation14"),
        cube: asset_server.load("cube/cube.glb#Scene0"),
        sword: asset_server.load("sword/sword.glb#Scene0"),
        gun: asset_server.load("gun/gun.glb#Scene0"),
    };
    commands.insert_resource(assets);
}

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_assets.in_base_set(StartupSet::PreStartup));
    }
}
