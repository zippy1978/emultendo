use crate::{cartridge::Cartridge, nes::NES};

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
    let mut nes = NES::new();
    nes.insert(cartridge);
    nes.reset();
    let mut tick_count = 0;
    nes.run(|_| {
        tick_count += 1;
    })
    .unwrap();
    assert_eq!(tick_count, 7)
}
