use std::sync::{Arc, RwLock};

use emulator::{start_emulator, EmulatorState};
use glium::backend::Facade;
use renderable::Renderable;
use ui::{display_window::DisplayWindow, cpu_window::CpuWindow};

mod emulator;
mod renderable;
mod support;
mod ui;

fn main() {
    // Emulator state
    let mut state = Arc::new(RwLock::new(EmulatorState::new()));

    // Start emulator
    start_emulator(&state);

    let mut system = support::init("Emultendo - debugger", 1024.0, 768.0);

    // Display window
    let mut display_window = DisplayWindow::new(20.0, 40.0);
    display_window
        .register_textures(system.display.get_context(), system.renderer.textures())
        .unwrap();

    // CPU window
    let cpu_window = CpuWindow::new(20.0, 580.0);

    // Main loop
    system.main_loop(move |_, ui, renderer, _display| {
        cpu_window.render(ui, renderer.textures(), &mut state);
        display_window.render(ui, renderer.textures(), &mut state);
    });
}
