use bevy::render::camera::ScalingMode;
use bevy::{prelude::*, TestPlugins};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use std::thread;

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
    app.world.spawn().insert_bundle(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical(5.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(0.0, 9.0, -6.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    app.world
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));

    app.world.spawn().insert_bundle(PointLightBundle {
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
            .add_plugin(RapierDebugRenderPlugin::default());
    } else {
        app.add_plugins(TestPlugins);
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
                app.update();
            }
            (self.check)(&app, res)
        }
    }
}
