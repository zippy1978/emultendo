use std::{cell::RefCell, rc::Rc};

use crate::bus::ppu_bus::PPUBus;

use self::{
    addr_register::AddrRegister, control_register::ControlRegister, mask_register::MaskRegister,
    scroll_register::ScrollRegister, status_register::StatusRegister,
};

mod addr_register;
#[cfg(test)]
mod addr_register_tests;

mod control_register;
#[cfg(test)]
mod control_register_tests;

mod mask_register;
mod scroll_register;
mod status_register;

/// PPU Error.
#[derive(Debug)]
pub enum PPUError {
    Unknown,
}

/// PPU.
#[derive(Debug, Clone)]
pub struct PPU {
    addr: AddrRegister,
    ctrl: ControlRegister,
    mask: MaskRegister,
    status: StatusRegister,
    scroll: ScrollRegister,
    oam_addr: u8,
    oam_data: [u8; 256],
    scanline: u16,
    cycles: usize,
    nmi_interrupt: bool,
    bus: Option<Rc<RefCell<PPUBus>>>,
}

impl PPU {
    /// Creates a PPU instance.
    pub fn new() -> Self {
        Self {
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            scroll: ScrollRegister::new(),
            oam_addr: 0,
            oam_data: [0; 64 * 4],
            scanline: 0,
            cycles: 0,
            nmi_interrupt: false,
            bus: None,
        }
    }

    pub fn nmi_interrupt(&self) -> bool {
        self.nmi_interrupt
    }

    pub fn ctrl(&self) -> &ControlRegister {
        &self.ctrl
    }

    pub fn bus(&self) -> &Option<Rc<RefCell<PPUBus>>> {
        &self.bus
    }

    pub fn oam_data(&self) -> &[u8; 256] {
        &self.oam_data
    }

    pub fn scroll(&self) -> &ScrollRegister {
        &self.scroll
    }

    /// Connects PPU to bus.
    pub fn connect_bus(&mut self, bus: &Rc<RefCell<PPUBus>>) {
        self.bus = Some(Rc::clone(bus));
    }

    /// Reads status register.
    pub fn read_status(&mut self) -> u8 {
        let data = self.status.snapshot();
        self.status.reset_vblank_status();
        self.addr.reset_latch();
        self.scroll.reset_latch();
        data
    }

    /// Writes to OAM address register.
    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    /// Writes to OAM data.
    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    /// Reads OAM data.
    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    /// Write to OAM DMA.
    pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    /// Writes to address register.
    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    /// Writes to control register.
    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = true;
        }
    }

    /// Writes to mask register.
    pub fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    /// Writes to scroll register.
    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    /// Increments VRAM address.
    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    /// Reads PPU data.
    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        if let Some(bus) = &self.bus {
            return bus.borrow_mut().read_data(addr);
        } else {
            panic!("PPU is not connected to bus");
        }
    }

    /// Write PPU data.
    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        if let Some(bus) = &self.bus {
            bus.borrow_mut().write_to_data(addr, value);
        } else {
            panic!("PPU is not connected to bus");
        }
        self.increment_vram_addr();
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        // TODO: consider checking opaque pixels of a sprite colliding with opaque pixels of background
        // for more accuracy
        (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
    }

    /// Processes next cycle.
    /// Returns true when a full new frame is ready
    pub fn tick(&mut self, cycles: u8) -> Result<bool, PPUError> {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            // Tests sprite 0 hit.
            if self.is_sprite_0_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }

            // End of scanline
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                // End of visible screen
                self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = false;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Poll NMI interrupt status.
    /// Sets to false after call.
    pub fn poll_nmi_status(&mut self) -> bool {
        let status = self.nmi_interrupt;
        self.nmi_interrupt = false;
        status
    }

}