use std::path::PathBuf;

use crate::cartridge::Cartridge;

use super::ppu_bus::PpuBus;

fn create_test_cartridge() -> Cartridge {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("res/test.nes");
    Cartridge::from_file(path).unwrap()
}

#[test]
fn test_read_data_chr_rom() {
    let mut cartridge = create_test_cartridge();
    cartridge.chr_rom = vec![0x06];
    
    let mut bus = PpuBus::new();
    bus.connect_cartridge(&cartridge);
    // Read twice to flush the internal buffer
    bus.read_data(0x0);
    assert_eq!(bus.read_data(0x0), 0x06);
}

#[test]
fn test_read_data_vram() {
    let mut bus = PpuBus::new();
    bus.write_to_vram(0x01, 0x05);
    // Read twice to flush the internal buffer
    bus.read_data(0x2001);
    assert_eq!(bus.read_data(0x2001), 0x05);
}

#[test]
#[should_panic]
fn test_read_data_forbidden() {
    let mut bus = PpuBus::new();
    bus.read_data(0x3000);
}