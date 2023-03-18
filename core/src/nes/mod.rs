use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
    bus::{cpu_bus::CPUBus, ppu_bus::PPUBus},
    cartridge::Cartridge,
    controller::Joypad,
    cpu::{CPUError, CPU},
    ppu::{PPUError, PPU},
    renderer::{frame::Frame, render_screen},
};

pub mod tools;

#[cfg(test)]
mod mod_tests;

/// NES Error.
#[derive(Debug)]
pub enum NESError {
    CPU(CPUError),
    PPU(PPUError),
}

impl From<CPUError> for NESError {
    fn from(value: CPUError) -> Self {
        Self::CPU(value)
    }
}

impl From<PPUError> for NESError {
    fn from(value: PPUError) -> Self {
        Self::PPU(value)
    }
}

/// NES console.
pub struct NES {
    cpu: CPU,
    cpu_bus: Rc<RefCell<CPUBus>>,
    ppu: Rc<RefCell<PPU>>,
    ppu_bus: Rc<RefCell<PPUBus>>,
    joypad1: Option<Rc<RefCell<Joypad>>>,
    joypad2: Option<Rc<RefCell<Joypad>>>,
    frame: Frame,
}

impl NES {
    pub fn new(joypad1: Option<Joypad>, joypad2: Option<Joypad>) -> Self {
        let mut this = Self {
            cpu: CPU::new(),
            cpu_bus: Rc::new(RefCell::new(CPUBus::new())),
            ppu: Rc::new(RefCell::new(PPU::new())),
            ppu_bus: Rc::new(RefCell::new(PPUBus::new())),
            joypad1: match joypad1 {
                Some(j) => Some(Rc::new(RefCell::new(j))),
                None => None,
            },
            joypad2: match joypad2 {
                Some(j) => Some(Rc::new(RefCell::new(j))),
                None => None,
            },
            frame: Frame::new(),
        };
        // Connects CPU bus to CPU
        this.cpu.connect_bus(&this.cpu_bus);
        // Connects PPU to CPU bus
        this.cpu_bus.borrow_mut().connect_ppu(&this.ppu);
        // Connects PPU bus to PPU
        this.ppu.borrow_mut().connect_bus(&this.ppu_bus);
        // Connects Joypad 1 to CPU bus
        if let Some(joypad1) = &this.joypad1 {
            this.cpu_bus.borrow_mut().connect_joypad1(&joypad1);
        }
        // Connects Joypad 2 to CPU bus
        if let Some(joypad2) = &this.joypad2 {
            this.cpu_bus.borrow_mut().connect_joypad2(&joypad2);
        }

        this
    }

    pub fn insert(&mut self, cartridge: Cartridge) {
        self.cpu_bus.borrow_mut().connect_cartridge(&cartridge);
        self.ppu_bus.borrow_mut().connect_cartridge(&cartridge);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    #[cfg(test)]
    pub fn start_at(&mut self, addr: u16) {
        self.cpu.program_counter = addr;
    }

    pub fn run<F1, F2>(
        &mut self,
        mut cpu_callback: F1,
        mut ppu_callback: F2,
    ) -> Result<(), NESError>
    where
        F1: FnMut(&mut CPU),
        F2: FnMut(&Frame, Option<&Rc<RefCell<Joypad>>>, Option<&Rc<RefCell<Joypad>>>),
    {
        let mut cont = true;

        while cont {
            let nmi_before = self.ppu.borrow_mut().nmi_interrupt();

            // CPU callback is called only on instruction change
            // Not for every cycle
            if self.cpu.instruction_changed() {
                cpu_callback(&mut self.cpu);
            }
            cont = self.cpu.tick()?;

            // PPU runs 3x faster than CPU
            self.ppu.borrow_mut().tick(3)?;

            let nmi_after = self.ppu.borrow_mut().nmi_interrupt();

            if !nmi_before && nmi_after {
                // Update frame
                render_screen(&self.ppu.borrow_mut(), &mut self.frame);

                ppu_callback(
                    &self.frame,
                    match &mut self.joypad1 {
                        Some(j) => Some(j),
                        None => None,
                    },
                    match &mut self.joypad2 {
                        Some(j) => Some(j),
                        None => None,
                    },
                );
            }
        }

        Ok(())
    }
}
