mod app;
mod game;
mod player;
mod render;
mod ui;
mod voxel;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Event loop exited with error: {:?}", e);
    }
}
