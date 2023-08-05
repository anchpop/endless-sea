use std::{thread, time::Duration};

use bevy::{app::PluginGroupBuilder, prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use endless_sea::asset_holder;

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
    use endless_sea::{animations, player, ui};

    app.add_plugins((
        asset_holder::Plugin,
        ui::Plugin,
        player::Plugin,
        animations::Plugin,
    ));

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

fn app(on_main_thread: bool) -> App {
    let mut app = App::new();

    if on_main_thread {
        app.add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ));
    } else {
        let time = Time::default();
        app.insert_resource(time)
            .add_plugins((
                TestPlugins,
                bevy::render::RenderPlugin {
                    wgpu_settings: bevy::render::settings::WgpuSettings {
                        backends: None,
                        ..default()
                    },
                },
                RapierPhysicsPlugin::<NoUserData>::default(),
            ))
            .insert_resource(RapierConfiguration {
                timestep_mode: TimestepMode::Fixed {
                    dt: 1.0 / TEST_FPS,
                    substeps: 1,
                },
                ..default()
            });
    }
    app.add_plugins(WorldInspectorPlugin::new());

    app
}

impl<A> Test<A> {
    pub fn run(self) {
        let on_main_thread = if on_main_thread() {
            println!("Test running on main thread, will display window");
            true
        } else {
            false
        };

        if on_main_thread {
            self.main_thread_app().run();
        } else {
            self.run_test();
        }
    }

    pub fn run_test(self) {
        let mut app = app(false);

        let res = (self.setup)(&mut app);

        for _ in 0..self.frames {
            // Update time manually for consistent time.delta()
            let mut time = app.world.resource_mut::<Time>();
            if let Some(last_update) = time.last_update() {
                time.update_with_instant(
                    last_update + Duration::from_secs_f32(1.0 / TEST_FPS),
                );
            } else {
                time.update();
            }
            // Run systems
            app.update();
        }
        (self.check)(&app, res)
    }

    pub fn main_thread_app(self) -> App {
        let mut app = app(true);
        let _res = (self.setup)(&mut app);
        (self.setup_graphics)(&mut app);
        app
    }
}

struct TestPlugins;

impl PluginGroup for TestPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy::core::TaskPoolPlugin::default())
            .add(bevy::core::TypeRegistrationPlugin::default())
            .add(bevy::core::FrameCountPlugin::default())
            .add(bevy::app::ScheduleRunnerPlugin::default())
            .add(bevy::window::WindowPlugin::default())
            .add(bevy::transform::TransformPlugin)
            .add(bevy::hierarchy::HierarchyPlugin)
            .add(bevy::diagnostic::DiagnosticsPlugin)
            .add(bevy::input::InputPlugin)
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(bevy::gilrs::GilrsPlugin::default())
            .add(bevy::render::texture::ImagePlugin::default())
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
