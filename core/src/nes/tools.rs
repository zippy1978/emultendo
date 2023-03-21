use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{cartridge::Cartridge, ppu::{frame::Frame, palette}};

/// Loads execution trace from file.
pub fn load_trace<P>(file: P) -> Result<Vec<String>, String>
where
    P: AsRef<Path>,
{
    let file = File::open(file).map_err(|e| e.to_string())?;
    let buf = BufReader::new(file);
    Ok(buf.lines().map(|l| l.unwrap()).collect())
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