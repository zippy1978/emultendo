use std::collections::HashMap;

use emultendo_core::{
    cartridge::Cartridge,
    controller::{Joypad, JoypadButton},
    nes::Nes,
    ppu::frame::Frame,
};

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, video::GLProfile};

fn main() {
    // Pixel scale
    let pixel_scale = 3.0;

    // Map Joypad buttons with keyboard
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, JoypadButton::DOWN);
    key_map.insert(Keycode::Up, JoypadButton::UP);
    key_map.insert(Keycode::Right, JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, JoypadButton::LEFT);
    key_map.insert(Keycode::Space, JoypadButton::SELECT);
    key_map.insert(Keycode::Return, JoypadButton::START);
    key_map.insert(Keycode::A, JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, JoypadButton::BUTTON_B);

    // Init SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    // Create main window
    // with a x3 scale
    let window = video_subsystem
        .window(
            "Emultendo",
            (Frame::WIDTH as f32 * pixel_scale) as u32,
            (Frame::HEIGHT as f32 * pixel_scale) as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    // Create canvas for frame rendering in main window
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(pixel_scale, pixel_scale).unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            Frame::WIDTH as u32,
            Frame::HEIGHT as u32,
        )
        .unwrap();

    // Create event pump
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Game filename
    let mut game_filename: Option<Box<String>> = None;

    loop {
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

        // Run
        nes.run(
            |_| {true},
            |ppu, joypad1, _| {
                // Update canvas with frame
                texture.update(None, &ppu.frame().borrow().data(), 256 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                // Run event loop
                let mut cont = true;
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => std::process::exit(0),

                        Event::KeyDown {
                            keycode: Some(Keycode::Tab),
                            ..
                        } => cont = false,

                        Event::DropFile { filename, .. } => {
                            game_filename = Some(Box::new(filename));
                            cont = false;
                        }

                        Event::KeyDown { keycode, .. } => {
                            if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                if let Some(joypad1) = &joypad1 {
                                    joypad1.borrow_mut().set_button_pressed_status(*key, true);
                                }
                            }
                        }
                        Event::KeyUp { keycode, .. } => {
                            if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                if let Some(joypad1) = &joypad1 {
                                    joypad1.borrow_mut().set_button_pressed_status(*key, false);
                                }
                            }
                        }

                        _ => { /* Do nothing */ }
                    }
                }

                // Continue ?
                cont
            },
        )
        .unwrap();
    }
}
