use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// The Material trait is very configurable, but comes with sensible defaults
/// for all methods. You only need to implement functions for features that need
/// non-default behavior. See the Material api docs for details!
impl Material for Custom {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct Custom {}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<Custom>::default());
    }
}
