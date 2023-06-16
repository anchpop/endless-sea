use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct AssetHolder {
    pub(crate) character: Handle<Scene>,
    pub(crate) character_run: Handle<AnimationClip>,
    pub(crate) character_idle: Handle<AnimationClip>,

    pub(crate) cube: Handle<Scene>,
    pub(crate) sword: Handle<Scene>,
    pub(crate) gun: Handle<Scene>,
}

pub(crate) fn load_assets(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let assets = AssetHolder {
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

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_assets.in_base_set(StartupSet::PreStartup));
    }
}
