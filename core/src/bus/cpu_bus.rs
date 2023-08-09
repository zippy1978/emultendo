use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, controller::Joypad, memory::Memory, ppu::Ppu};

//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|
const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

/// NES CPU connection bus.
#[derive(Debug, Clone)]
pub struct CpuBus {
    vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: Option<Rc<RefCell<Ppu>>>,
    joypad1: Option<Rc<RefCell<Joypad>>>,
    joypad2: Option<Rc<RefCell<Joypad>>>,
}

impl CpuBus {
    /// Creates a bus.
    pub fn new() -> Self {
        CpuBus {
            vram: [0; 2048],
            prg_rom: vec![],
            ppu: None,
            joypad1: None,
            joypad2: None,
        }
    }

    /// Checks if a cartridge is connected
    pub fn cartridge_connected(&self) -> bool {
        self.prg_rom.len() > 0
    }

    /// Connects a cartridge to the bus.
    pub fn connect_cartridge(&mut self, cartridge: &Cartridge) {
        self.prg_rom = cartridge.prg_rom.clone();
    }

    /// Connects PPU to the bus.
    pub fn connect_ppu(&mut self, ppu: &Rc<RefCell<Ppu>>) {
        self.ppu = Some(Rc::clone(ppu));
    }

    /// Connects Joypad 1 to the bus
    pub fn connect_joypad1(&mut self, joypad: &Rc<RefCell<Joypad>>) {
        self.joypad1 = Some(Rc::clone(joypad));
    }

    /// Connects Joypad 2 to the bus
    pub fn connect_joypad2(&mut self, joypad: &Rc<RefCell<Joypad>>) {
        self.joypad2 = Some(Rc::clone(joypad));
    }

    /// Reads PROG_ROM.
    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // Mirror if needed
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn poll_nmi_status(&self) -> bool {
        if let Some(ppu) = &self.ppu {
            return ppu.borrow_mut().poll_nmi_status();
        } else {
            panic!("PPU is not connected to CPU bus");
        }
    }
}

impl Memory for CpuBus {
    /// Reads memory address.
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // Commented to keep the CPU trace working
                //panic!("attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().read_status();
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2004 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().read_oam_data();
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2007 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().read_data();
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x4000..=0x4015 => {
                //Ignore APU
                0
            }

            0x4016 => {
                if let Some(joypad1) = &self.joypad1 {
                    joypad1.borrow_mut().read()
                } else {
                    0
                }
            }
            0x4017 => {
                if let Some(joypad2) = &self.joypad2 {
                    joypad2.borrow_mut().read()
                } else {
                    0
                }
            }

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            PRG_ROM..=PRG_ROM_END => self.read_prg_rom(addr),
            _ => {
                // Ignore access
                0
            }
        }
    }

    /// Writes to memory address.
    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_ctrl(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2001 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_mask(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2002 => panic!("attempt to write to PPU status register"),
            0x2003 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_oam_addr(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2004 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_oam_data(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2005 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_scroll(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2006 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_ppu_addr(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2007 => {
                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_to_data(data);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x4000..=0x4013 | 0x4015 => {
                // Ignore APU
            }

            0x4016 => {
                if let Some(joypad1) = &self.joypad1 {
                    joypad1.borrow_mut().write(data)
                }
            }

            0x4017 => {
                if let Some(joypad2) = &self.joypad2 {
                    joypad2.borrow_mut().write(data)
                }
            }

            // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                if let Some(ppu) = &self.ppu {
                    return ppu.borrow_mut().write_oam_dma(&buffer);
                } else {
                    panic!("PPU is not connected to CPU bus");
                }
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            PRG_ROM..=PRG_ROM_END => {
                // Attempt to write to Cartridge ROM space
                panic!("attempt to write to cartridge PRG ROM space");
            }
            _ => {
                // Ignore access
            }
        }
    }
}
