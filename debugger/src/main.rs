use std::{
    sync::{Arc, RwLock},
    thread,
};

use cpu_window::CpuWindow;
use emultendo_core::{cartridge::Cartridge, controller::Joypad, nes::Nes, ppu::frame::Frame};
use glium::backend::Facade;
use display_window::DisplayWindow;

mod display_window;
mod cpu_window;
mod support;

fn main() {
    let current_frame = Arc::new(RwLock::new(Frame::new()));
    let ui_frame = current_frame.clone();

    // Run emulator in thread
    thread::spawn(move || {
        let current_frame = &current_frame.clone();
        let game_filename = Some(Box::new("../games/smario.nes".to_string()));
        //let game_filename: Option<Box<String>> = None;

        // Create console
        // plug only joypad1, other Super Mario does not work
        let mut nes = Nes::new(Some(Joypad::new()), None);

        // Load game to cartridge (if game file)
        // then insert cartridge and reset
        if let Some(game_filename) = &game_filename {
            let cartridge = Cartridge::from_file(game_filename.as_ref()).unwrap();
            nes.insert(cartridge);
            nes.reset();
        }

        nes.run(
            |_| {},
            |frame, _, _| {
                let mut frame_lock = current_frame.write().unwrap();
                *frame_lock = frame.clone();
                true
            },
        )
    });

    let mut system = support::init(file!());

    // Display window
    let mut display_window = DisplayWindow::new();
    display_window
        .register_textures(system.display.get_context(), system.renderer.textures())
        .unwrap();

    // CPU window
    let cpu_window = CpuWindow::new();

    system.main_loop(move |_, ui, renderer, display| {
        display_window
            .update(&ui_frame.read().unwrap(), renderer.textures())
            .unwrap();

        display_window.render(ui);
        cpu_window.render(ui);
    });

}
