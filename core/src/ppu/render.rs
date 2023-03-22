use crate::{bus::ppu_bus::PPUBus, cartridge::Mirroring, ppu::PPU};

use super::{
    frame::Frame,
    palette::{self, bg_palette, sprite_palette},
    rect::Rect,
    status_register::StatusRegister,
};

fn render_name_table_sync(
    ppu: &PPU,
    bus: &PPUBus,
    frame: &mut Frame,
    name_table: &[u8],
    view_port: Rect,
    shift_x: isize,
    shift_y: isize,
) -> bool {
    let cycles = ppu.cycle as usize;
    let scanline = ppu.scanline as usize;

    // Don't draw background if disabled
    let background_visible =
        ppu.mask.show_background() && (ppu.mask.leftmost_8pxl_background() || cycles >= 8);
    if !background_visible {
        return false;
    }

    // Pixel must be in viewport
    if cycles < view_port.x1
        || cycles >= view_port.x2
        || scanline < view_port.y1
        || scanline >= view_port.y2
    {
        return false;
    }

    let mut sprite_zero_hit = ppu.status().contains(StatusRegister::SPRITE_ZERO_HIT);

    let bank = ppu.ctrl.bknd_pattern_addr();

    let attribute_table = &name_table[0x3c0..0x400];

    // Width = 256 / 8 = 32
    // Height = 240 / 8 = 30
    let tile_row = (scanline / 8) as usize;
    let tile_column = (cycles / 8) as usize;

    let tile_addr = tile_row * 32 + tile_column;

    let tile_idx = name_table[tile_addr as usize] as u16;
    let tile =
        &bus.chr_rom()[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];
    let palette = bg_palette(&bus, attribute_table, tile_column, tile_row);

    // Determine tile matching pixel
    let tile_x = 7 - (cycles % 8);
    let tile_y = scanline % 8;

    let upper = tile[tile_y] >> tile_x;
    let lower = tile[tile_y + 8] >> tile_x;
    let value = (1 & lower) << 1 | (1 & upper);
    let rgb = match value {
        0 => palette::SYSTEM_PALETTE[bus.palette_table()[0] as usize],
        1 => palette::SYSTEM_PALETTE[palette[1] as usize],
        2 => palette::SYSTEM_PALETTE[palette[2] as usize],
        3 => palette::SYSTEM_PALETTE[palette[3] as usize],
        _ => panic!("can't be"),
    };

    if cycles >= view_port.x1
        && cycles < view_port.x2
        && scanline >= view_port.y1
        && scanline < view_port.y2
    {
        let pixel_x = (shift_x + cycles as isize) as usize;
        let pixel_y = (shift_y + scanline as isize) as usize;

        // Test sprite zero hit
        if !sprite_zero_hit && value != 0 && sprite_zero_hit_at(ppu, bus, pixel_x, pixel_y) {
            sprite_zero_hit = true;
        }

        frame.set_pixel(pixel_x, pixel_y, rgb);
    }

    sprite_zero_hit
}

fn sprite_zero_hit_at(ppu: &PPU, bus: &PPUBus, test_x: usize, test_y: usize) -> bool {
    // No hit if sprites are not visible
    let sprites_visible =
        ppu.mask.show_sprites() && (ppu.mask.leftmost_8pxl_sprite() || test_x >= 8);
    if !sprites_visible {
        return false;
    }

    let tile_idx = ppu.oam_data[1] as u16;
    let tile_x = ppu.oam_data[3] as usize;
    let tile_y = ppu.oam_data[0] as usize;

    let flip_vertical = ppu.oam_data[2] >> 7 & 1 == 1;
    let flip_horizontal = ppu.oam_data[2] >> 6 & 1 == 1;
    let bank: u16 = ppu.ctrl.sprt_pattern_addr();

    let tile =
        &bus.chr_rom()[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];

    for y in 0..=7 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];
        for x in (0..=7).rev() {
            let value = (1 & lower) << 1 | (1 & upper);
            upper = upper >> 1;
            lower = lower >> 1;
            if value != 0 {
                let (pixel_x, pixel_y) = match (flip_horizontal, flip_vertical) {
                    (false, false) => (tile_x + x, tile_y + y),
                    (true, false) => (tile_x + 7 - x, tile_y + y),
                    (false, true) => (tile_x + x, tile_y + 7 - y),
                    (true, true) => (tile_x + 7 - x, tile_y + 7 - y),
                };

                if test_x == pixel_x && test_y == pixel_y {
                    return true;
                }
            }
        }
    }
    false
}

pub(crate) fn render_background_sync(ppu: &PPU, frame: &mut Frame) -> bool {
    let bus = match &ppu.bus {
        Some(b) => b.borrow_mut(),
        None => panic!("PPU is not connected to bus"),
    };

    let mut sprite_zero_hit = false;

    // Get scroll position
    let scroll_x = (ppu.scroll.scroll_x) as usize;
    let scroll_y = (ppu.scroll.scroll_y) as usize;

    // Determine main and second table
    let (main_nametable, second_nametable) = match (&bus.mirroring(), ppu.ctrl.nametable_addr()) {
        (Mirroring::Vertical, 0x2000)
        | (Mirroring::Vertical, 0x2800)
        | (Mirroring::Horizontal, 0x2000)
        | (Mirroring::Horizontal, 0x2400) => (&bus.vram()[0..0x400], &bus.vram()[0x400..0x800]),
        (Mirroring::Vertical, 0x2400)
        | (Mirroring::Vertical, 0x2C00)
        | (Mirroring::Horizontal, 0x2800)
        | (Mirroring::Horizontal, 0x2C00) => (&bus.vram()[0x400..0x800], &bus.vram()[0..0x400]),
        (_, _) => {
            panic!("unsupported mirroring type {:?}", bus.mirroring());
        }
    };

    // Top left
    sprite_zero_hit = sprite_zero_hit
        || render_name_table_sync(
            ppu,
            &bus,
            frame,
            main_nametable,
            Rect::new(scroll_x, scroll_y, Frame::WIDTH, Frame::HEIGHT),
            -(scroll_x as isize),
            -(scroll_y as isize),
        );

    // Bottom left
    sprite_zero_hit = sprite_zero_hit
        || render_name_table_sync(
            ppu,
            &bus,
            frame,
            if matches!(bus.mirroring(), Mirroring::Vertical) {
                main_nametable
            } else {
                second_nametable
            },
            Rect::new(scroll_x, 0, Frame::WIDTH, Frame::HEIGHT),
            -(scroll_x as isize),
            (Frame::HEIGHT - scroll_y) as isize,
        );

    // Top right
    sprite_zero_hit = sprite_zero_hit
        || render_name_table_sync(
            ppu,
            &bus,
            frame,
            if matches!(bus.mirroring(), Mirroring::Vertical) {
                second_nametable
            } else {
                main_nametable
            },
            Rect::new(0, scroll_y, scroll_x, Frame::HEIGHT),
            (Frame::WIDTH - scroll_x) as isize,
            -(scroll_y as isize),
        );

    // Bottom right
    sprite_zero_hit = sprite_zero_hit
        || render_name_table_sync(
            ppu,
            &bus,
            frame,
            second_nametable,
            Rect::new(0, 0, Frame::WIDTH, Frame::HEIGHT),
            (Frame::WIDTH - scroll_x) as isize,
            (Frame::HEIGHT - scroll_y) as isize,
        );

    sprite_zero_hit
}

pub(crate) fn render_sprites(ppu: &PPU, frame: &mut Frame) {
    let bus = match &ppu.bus {
        Some(b) => b.borrow_mut(),
        None => panic!("PPU is not connected to bus"),
    };

    // Sprites
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let tile_x = ppu.oam_data[i + 3] as usize;
        let tile_y = ppu.oam_data[i] as usize;

        let flip_vertical = ppu.oam_data[i + 2] >> 7 & 1 == 1;
        let flip_horizontal = ppu.oam_data[i + 2] >> 6 & 1 == 1;
        let pallette_idx = ppu.oam_data[i + 2] & 0b11;
        let sprite_palette = sprite_palette(&bus, pallette_idx);
        let bank: u16 = ppu.ctrl.sprt_pattern_addr();

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
