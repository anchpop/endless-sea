use std::{thread, time::Duration};

use bevy::{app::PluginGroupBuilder, prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

use crate::asset_holder;

pub const TEST_FPS: f32 = 144.0;

pub fn on_main_thread() -> bool {
    matches!(thread::current().name(), Some("main"))
}

pub struct Test<A> {
    pub setup: fn(&mut App) -> A,
    pub setup_graphics: fn(&mut App),
    pub frames: u64,
    pub check: fn(&App, A),
}

pub fn default_setup_graphics(app: &mut App) {
    use crate::{animations, player, ui};

    app.add_plugin(asset_holder::Plugin)
        .add_plugin(ui::Plugin)
        .add_plugin(player::Plugin)
        .add_plugin(animations::Plugin);

    app.world.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::FixedVertical(5.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(0.0, 9.0, -6.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        player::PlayerCamera {},
    ));

    app.world
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));

    app.world.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn app() -> (App, bool) {
    let mut app = App::new();
    let on_main_thread = if on_main_thread() {
        println!("Test running on main thread, will display window");
        true
    } else {
        println!("Test not running on main thread, will run headlessly");
        false
    };

    if on_main_thread {
        app.add_plugins(DefaultPlugins)
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(RapierConfiguration {
                timestep_mode: TimestepMode::Fixed {
                    dt: 1.0 / TEST_FPS,
                    substeps: 1,
                },
                ..default()
            })
            .add_plugin(RapierDebugRenderPlugin::default());
    } else {
        let time = Time::default();
        app.insert_resource(time)
            .insert_resource(bevy::render::settings::WgpuSettings {
                backends: None,
                ..default()
            })
            .add_plugins(TestPlugins)
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
    }
    app.add_plugin(WorldInspectorPlugin::new());

    (app, on_main_thread)
}

impl<A> Test<A> {
    pub fn run(self) {
        let (mut app, on_main_thread) = app();
        if on_main_thread {
            (self.setup_graphics)(&mut app);
        }

        let res = (self.setup)(&mut app);

        if on_main_thread {
            app.run();
        } else {
            for _ in 0..self.frames {
                // Update time manually for consistent time.delta()
                let mut time = app.world.resource_mut::<Time>();
                if let Some(last_update) = time.last_update() {
                    time.update_with_instant(
                        last_update
                            + Duration::from_secs_f32((1.0 / TEST_FPS) as f32),
                    );
                } else {
                    time.update();
                }
                // Run systems
                app.update();
            }
            (self.check)(&app, res)
        }
    }
}

struct TestPlugins;

impl PluginGroup for TestPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy::core::CorePlugin::default())
            .add(bevy::app::ScheduleRunnerPlugin::default())
            .add(bevy::window::WindowPlugin::default())
            .add(bevy::transform::TransformPlugin)
            .add(bevy::hierarchy::HierarchyPlugin)
            .add(bevy::diagnostic::DiagnosticsPlugin)
            .add(bevy::input::InputPlugin)
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(bevy::gilrs::GilrsPlugin::default())
            .add(bevy::render::RenderPlugin::default())
    }
}

pub fn spawn_floor_beneath_capsule(app: &mut App, capsule_id: Entity) {
    let transform = *app.world.get::<Transform>(capsule_id).unwrap();
    let collider = app.world.get::<Collider>(capsule_id).unwrap().clone();
    let capsule = collider.as_capsule().unwrap();
    app.world.spawn(Collider::cuboid(0.5, 0.5, 0.5)).insert((
        TransformBundle::from(Transform {
            translation: Vec3::ZERO
                - transform.translation
                - Vec3::Y * capsule.height(),
            scale: Vec3::new(100.0, 1.0, 100.0),
            ..Default::default()
        }),
        Name::new("floor"),
    ));
}
