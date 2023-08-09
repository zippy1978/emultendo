use std::{cell::RefCell, rc::Rc};

use crate::bus::ppu_bus::PpuBus;

use self::{
    addr_register::AddrRegister,
    control_register::ControlRegister,
    frame::Frame,
    mask_register::MaskRegister,
    render::{render_background_sync, render_sprites},
    scroll_register::ScrollRegister,
    status_register::StatusRegister,
};

mod addr_register;
#[cfg(test)]
mod addr_register_tests;

mod control_register;
#[cfg(test)]
mod control_register_tests;

mod mask_register;
mod scroll_register;
#[cfg(test)]
mod scroll_register_tests;
mod status_register;
#[cfg(test)]
mod status_register_tests;

pub mod frame;
pub mod palette;
pub mod rect;
pub mod render;

/// PPU Error.
#[derive(Debug)]
pub enum PpuError {
    Unknown,
}

/// PPU.
#[derive(Debug, Clone)]
pub struct Ppu {
    addr: AddrRegister,
    ctrl: ControlRegister,
    mask: MaskRegister,
    status: StatusRegister,
    scroll: ScrollRegister,
    oam_addr: u8,
    oam_data: [u8; 256],
    scanline: u16,
    cycle: usize,
    nmi_interrupt: bool,
    bus: Option<Rc<RefCell<PpuBus>>>,
    frame: Rc<RefCell<Frame>>,
}

impl Ppu {
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
            cycle: 0,
            nmi_interrupt: false,
            bus: None,
            frame: Rc::new(RefCell::new(Frame::new())),
        }
    }

    pub fn frame(&self) -> &Rc<RefCell<Frame>> {
        &self.frame
    }

    pub fn nmi_interrupt(&self) -> bool {
        self.nmi_interrupt
    }

    pub fn ctrl(&self) -> &ControlRegister {
        &self.ctrl
    }

    pub fn bus(&self) -> &Option<Rc<RefCell<PpuBus>>> {
        &self.bus
    }

    pub fn oam_data(&self) -> &[u8; 256] {
        &self.oam_data
    }

    pub fn scroll(&self) -> &ScrollRegister {
        &self.scroll
    }

    pub fn status(&self) -> &StatusRegister {
        &self.status
    }

    pub fn cycles(&self) -> usize {
        self.cycle
    }

    pub fn scanline(&self) -> u16 {
        self.scanline
    }

    /// Connects PPU to bus.
    pub fn connect_bus(&mut self, bus: &Rc<RefCell<PpuBus>>) {
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

    /// Processes next cycle.
    /// Returns true when a full scanline is ready
    pub fn tick(&mut self) -> Result<bool, PpuError> {
        // Render background in sync
        if render_background_sync(self, &mut self.frame.borrow_mut()) {
            self.status.set_sprite_zero_hit(true);
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            // End of scanline
            self.cycle = self.cycle - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                // End of visible screen

                // Render sprites at once at the end of the screen
                render_sprites(self, &mut self.frame.borrow_mut());

                self.status.set_vblank_status(true);
                if self.ctrl.generate_vblank_nmi() {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = false;
                self.status.reset_vblank_status();
                self.status.set_sprite_zero_hit(false);
                self.status.set_sprite_overflow(false);

                // Seems that name table addr must be reset after each frame
                // Source: https://archive.nes.science/nesdev-forums/f3/t12185.xhtml
                // But is it the proper way ?
                self.ctrl.set(ControlRegister::NAMETABLE1, false);
                self.ctrl.set(ControlRegister::NAMETABLE2, false);
            }

            return Ok(true);
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
