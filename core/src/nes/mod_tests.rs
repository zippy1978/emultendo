use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{cartridge::Cartridge, cpu::trace::Trace, nes::NES};

use super::tools::load_trace;

fn run_test_suite(cartridge_file: &str, log_file: &str, start_at: Option<u16>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cartridge_file_path = path.clone();
    cartridge_file_path.push(cartridge_file);
    let mut log_file_path = path.clone();
    log_file_path.push(log_file);
    let expected = load_trace(log_file_path).unwrap();
    let cartridge = Cartridge::from_file(cartridge_file_path).unwrap();
    let mut nes = NES::new(None, None);
    nes.insert(cartridge);
    nes.reset();
    if let Some(start_at) = start_at {
        nes.start_at(start_at);
    }
    let mut counter = 0;
    nes.run(|cpu| {
        assert_eq!(
            expected[counter].split(" PPU").collect::<Vec<&str>>()[0],
            cpu.trace(),
            "failure at line {} of {}, previous instruction: {}",
            counter + 1,
            log_file,
            expected[counter - 1].split(" PPU").collect::<Vec<&str>>()[0]
        );
        counter += 1;
    }, |_ ,_, _| {})
    .unwrap();
}

#[test]
fn test_run() {
    // Push 5 to a, then push 2 to a (that is: 2 instructions)
    let code = vec![0xa9, 0x05, 0xa9, 0x02, 0x00];

    let mut prg_rom: [u8; 0xFFFF] = [0; 0xFFFF];
    prg_rom[0..code.len()].copy_from_slice(&code[..]);

    // Set program start at address 0xFFFC (0x8000)
    let hi = (0x8000 >> 8) as u8;
    let lo = (0x8000 & 0xff) as u8;
    prg_rom[0xFFFC - 0x8000] = lo;
    prg_rom[0xFFFC + 1 - 0x8000] = hi;

    let cartridge = Cartridge {
        prg_rom: prg_rom.to_vec(),
        chr_rom: vec![],
        mapper: 0,
        screen_mirroring: crate::cartridge::Mirroring::FourScreen,
    };
    let mut nes = NES::new(None, None);
    nes.insert(cartridge);
    nes.reset();
    let mut inst_count = 0;
    nes.run(|_| {
        inst_count += 1;
    }, |_,_, _| {})
    .unwrap();
    assert_eq!(inst_count, 3)
}

/*#[test]
fn test_nestest() {
    run_test_suite("res/nestest.nes", "res/nestest.log", Some(0xC000));
}*/
