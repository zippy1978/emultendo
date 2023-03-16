use crate::cartridge::{Cartridge, Mirroring};

/// NES PPU connection bus.
#[derive(Debug, Clone)]
pub struct PPUBus {
    chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    vram: [u8; 2048],
    mirroring: Mirroring,
    internal_data_buf: u8,
}

impl PPUBus {
    /// Creates a bus.
    pub fn new() -> Self {
        Self {
            chr_rom: vec![],
            mirroring: Mirroring::Vertical,
            vram: [0; 2048],
            palette_table: [0; 32],
            internal_data_buf: 0,
        }
    }

    #[cfg(test)]
    pub fn write_to_vram(&mut self, addr: u16, value: u8) {
        self.vram[addr as usize] = value;
    }

    pub fn vram(&self) -> &[u8; 2048] {
        &self.vram
    }

    pub fn chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }

    pub fn palette_table(&self) -> &[u8; 32] {
        &self.palette_table
    }

    pub fn mirroring(&self) -> &Mirroring {
        &self.mirroring
    }

    /// Connects a cartridge to the bus.
    pub fn connect_cartridge(&mut self, cartridge: &Cartridge) {
        self.chr_rom = cartridge.chr_rom.clone();
        self.mirroring = cartridge.screen_mirroring;
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    /// Reads data.
    pub fn read_data(&mut self, addr: u16) -> u8 {
        match addr {
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => panic!(
                "address space 0x3000..0x3eff is not expected to be used, requested = {} ",
                addr
            ),
            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    /// Writes data.
    pub fn write_to_data(&mut self, addr: u16, value: u8) {
        match addr {
            0..=0x1fff => panic!("attempt to write to CHR ROM space {}", addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3eff => panic!("address {} shouldn't be used", addr),

            // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.palette_table[(add_mirror - 0x3f00) as usize] = value;
            }
            0x3f00..=0x3fff => {
                self.palette_table[(addr - 0x3f00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }
}
