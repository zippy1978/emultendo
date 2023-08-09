use std::sync::{Arc, RwLock};

use emulator::{start_emulator, state::EmulatorState};
use glium::backend::Facade;
use widget::Widget;
use window::{
    cartridge::CartridgeWindow, control::ControlWindow, cpu::CpuWindow, display::DisplayWindow,
    ppu::PpuWindow,
};

mod emulator;
mod widget;
mod support;
mod window;

fn main() {
    // Emulator state
    let mut state = Arc::new(RwLock::new(EmulatorState::new()));

    // Start emulator
    start_emulator(&state);

    let mut system = support::init("Emultendo - debugger", 1280.0, 768.0);

    // Display window
    let mut display_window = DisplayWindow::new(20.0, 40.0);
    display_window
        .register_textures(system.display.get_context(), system.renderer.textures())
        .unwrap();

    // CPU window
    let cpu_window = CpuWindow::new(20.0, 580.0);

    // PPU window
    let mut ppu_window = PpuWindow::new(600.0, 40.0);
    ppu_window
        .register_textures(system.display.get_context(), system.renderer.textures())
        .unwrap();

    // Cartridge window
    let cartridge_window = CartridgeWindow::new(350.0, 580.0);

    // Control window
    let control_window = ControlWindow::new(20.0, 700.0);

    // Main loop
    system.main_loop(move |_, ui, renderer, _display| {
        cpu_window.render(ui, renderer.textures(), &mut state);
        ppu_window.render(ui, renderer.textures(), &mut state);
        cartridge_window.render(ui, renderer.textures(), &mut state);
        control_window.render(ui, renderer.textures(), &mut state);
        display_window.render(ui, renderer.textures(), &mut state);
    });
}
