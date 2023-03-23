use crate::{bus::cpu_bus::CpuBus, cartridge::Cartridge, memory::Memory};

#[test]
fn test_ram_read_write() {
    let mut bus = CpuBus::new();
    bus.mem_write(0x0001, 0x01);
    assert_eq!(bus.mem_read(0x0001), 0x01);
    bus.mem_write_u16(0x0200, 0x01);
    assert_eq!(bus.mem_read_u16(0x0200), 0x01);
}

#[test]
fn test_prg_rom_read() {
    let mut bus = CpuBus::new();
    let cartridge = Cartridge {
        prg_rom: vec![0xd8, 0x00],
        chr_rom: vec![],
        mapper: 0,
        screen_mirroring: crate::cartridge::Mirroring::FourScreen,
    };
    bus.connect_cartridge(&cartridge);
    assert_eq!(bus.mem_read_u16(0x8000), 0xd8);
}

#[test]
#[should_panic]
fn test_prg_rom_not_writable() {
    let mut bus = CpuBus::new();
    bus.mem_write_u16(0x8000, 0x001);
    assert_eq!(bus.mem_read_u16(0x8000), 0);
}
