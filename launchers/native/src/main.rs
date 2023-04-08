use std::io::Cursor;

use bevy::{prelude::*, window::Window, winit::WinitWindows};
use winit::window::Icon;

fn main() {
    let mut app = endless_sea::app();

    info!("Starting launcher: Native");
    //app.add_startup_system(set_window_icon);
    app.run();
}
