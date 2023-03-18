use crate::{
    cartridge::{Cartridge, Mirroring},
    ppu::PPU, bus::ppu_bus::PPUBus,
};

use self::{
    frame::Frame,
    palette::{bg_palette, sprite_palette},
    rect::Rect,
};

pub mod frame;
pub mod palette;
pub mod rect;

fn render_name_table(
    ppu: &PPU,
    bus: &PPUBus,
    frame: &mut Frame,
    name_table: &[u8],
    view_port: Rect,
    shift_x: isize,
    shift_y: isize,
) {

    let bank = ppu.ctrl().bknd_pattern_addr();

    let attribute_table = &name_table[0x3c0..0x400];

    for i in 0..0x3c0 {
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile_idx = name_table[i] as u16;
        let tile =
            &bus.chr_rom()[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];
        let palette = bg_palette(&bus, attribute_table, tile_column, tile_row);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALETTE[bus.palette_table()[0] as usize],
                    1 => palette::SYSTEM_PALETTE[palette[1] as usize],
                    2 => palette::SYSTEM_PALETTE[palette[2] as usize],
                    3 => palette::SYSTEM_PALETTE[palette[3] as usize],
                    _ => panic!("can't be"),
                };
                let pixel_x = tile_column * 8 + x;
                let pixel_y = tile_row * 8 + y;

                if pixel_x >= view_port.x1
                    && pixel_x < view_port.x2
                    && pixel_y >= view_port.y1
                    && pixel_y < view_port.y2
                {
                    frame.set_pixel(
                        (shift_x + pixel_x as isize) as usize,
                        (shift_y + pixel_y as isize) as usize,
                        rgb,
                    );
                }
            }
        }
    }
}

/// Renders screen from PPU state to a provided frame.
pub fn render_screen(ppu: &PPU, frame: &mut Frame) {
    let bus = match ppu.bus() {
        Some(b) => b.borrow_mut(),
        None => panic!("PPU is not connected to bus"),
    };

    // Get scroll position
    let scroll_x = (ppu.scroll().scroll_x) as usize;
    let scroll_y = (ppu.scroll().scroll_y) as usize;

    // Determine main and second table
    let (main_nametable, second_nametable) = match (&bus.mirroring(), ppu.ctrl().nametable_addr()) {
        (Mirroring::Vertical, 0x2000)
        | (Mirroring::Vertical, 0x2800)
        | (Mirroring::Horizontal, 0x2000)
        | (Mirroring::Horizontal, 0x2400) => (&bus.vram()[0..0x400], &bus.vram()[0x400..0x800]),
        (Mirroring::Vertical, 0x2400)
        | (Mirroring::Vertical, 0x2C00)
        | (Mirroring::Horizontal, 0x2800)
        | (Mirroring::Horizontal, 0x2C00) => (&bus.vram()[0x400..0x800], &bus.vram()[0..0x400]),
        (_, _) => {
            panic!("Not supported mirroring type {:?}", bus.mirroring());
        }
    };

    // Background
    render_name_table(
        ppu,
        &bus,
        frame,
        main_nametable,
        Rect::new(scroll_x, scroll_y, Frame::WIDTH, Frame::HEIGHT),
        -(scroll_x as isize),
        -(scroll_y as isize),
    );
    if scroll_x > 0 {
        render_name_table(
            ppu,
            &bus,
            frame,
            second_nametable,
            Rect::new(0, 0, scroll_x, Frame::HEIGHT),
            (Frame::WIDTH - scroll_x) as isize,
            0,
        );
    } else if scroll_y > 0 {
        render_name_table(
            ppu,
            &bus,
            frame,
            second_nametable,
            Rect::new(0, 0, Frame::WIDTH, scroll_y),
            0,
            (Frame::HEIGHT - scroll_y) as isize,
        );
    }

    // Sprites
    for i in (0..ppu.oam_data().len()).step_by(4).rev() {
        let tile_idx = ppu.oam_data()[i + 1] as u16;
        let tile_x = ppu.oam_data()[i + 3] as usize;
        let tile_y = ppu.oam_data()[i] as usize;

        let flip_vertical = ppu.oam_data()[i + 2] >> 7 & 1 == 1;
        let flip_horizontal = ppu.oam_data()[i + 2] >> 6 & 1 == 1;
        let pallette_idx = ppu.oam_data()[i + 2] & 0b11;
        let sprite_palette = sprite_palette(&bus, pallette_idx);
        let bank: u16 = ppu.ctrl().sprt_pattern_addr();

        let tile =
            &bus.chr_rom()[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
            'ololo: for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => continue 'ololo, // skip coloring the pixel
                    1 => palette::SYSTEM_PALETTE[sprite_palette[1] as usize],
                    2 => palette::SYSTEM_PALETTE[sprite_palette[2] as usize],
                    3 => palette::SYSTEM_PALETTE[sprite_palette[3] as usize],
                    _ => panic!("can't be"),
                };
                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                }
            }
        }
    }
}

/// Renders cartridge tiles of a given bank (0 or 1) to a frame.
pub fn render_tiles_bank(cartridge: &Cartridge, bank: usize) -> Frame {
    let chr_rom = &cartridge.chr_rom;

    assert!(bank <= 1);

    let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALETTE[0x01],
                    1 => palette::SYSTEM_PALETTE[0x23],
                    2 => palette::SYSTEM_PALETTE[0x27],
                    3 => palette::SYSTEM_PALETTE[0x30],
                    _ => panic!("unreachable case"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }
        tile_x += 10;
    }
    frame
}
