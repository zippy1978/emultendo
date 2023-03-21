use std::{collections::HashMap, path::Path};

use emultendo_core::{
    cartridge::Cartridge,
    controller::{Joypad, JoypadButton},
    nes::{NES, tools::render_tiles_bank}, ppu::frame::Frame,
};
use image::RgbImage;

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

fn dump_tiles(file: &str) {
    let game_name = Path::new(file).file_stem().unwrap().to_str().unwrap();
    let cartridge = Cartridge::from_file(file).unwrap();
    for bank in 0..2 {
        let frame = render_tiles_bank(&cartridge, bank);
        frame_to_file(&frame, format!("{}-tiles-{}.png", game_name, bank).as_str());
    }
}

fn frame_to_file(frame: &Frame, file: &str) {
    let img = RgbImage::from_raw(
        Frame::WIDTH as u32,
        Frame::HEIGHT as u32,
        frame.data().to_vec(),
    )
    .unwrap();
    img.save(file).unwrap();
}

fn main() {
    let game = "games/ice.nes";

    //dump_tiles(game);

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, JoypadButton::DOWN);
    key_map.insert(Keycode::Up, JoypadButton::UP);
    key_map.insert(Keycode::Right, JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, JoypadButton::LEFT);
    key_map.insert(Keycode::Space, JoypadButton::SELECT);
    key_map.insert(Keycode::Return, JoypadButton::START);
    key_map.insert(Keycode::A, JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, JoypadButton::BUTTON_B);

    let cartridge = Cartridge::from_file(game).unwrap();

    // Plug only joypad1, other Super Mario does not work
    let mut nes = NES::new(Some(Joypad::new()), None);
    nes.insert(cartridge);
    nes.reset();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(game, (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    nes.run(
        |_| {},
        |frame, joypad1, joypad2| {
            //frame_to_file(&frame, "frame.png");
            texture.update(None, &frame.data(), 256 * 3).unwrap();

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();

            // Run event loop
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),

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
        },
    )
    .unwrap();
}
