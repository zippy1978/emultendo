use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::{cpu_bus::CpuBus, ppu_bus::PpuBus},
    cartridge::Cartridge,
    controller::Joypad,
    cpu::{CpuError, Cpu},
    ppu::{frame::Frame, PpuError, Ppu},
};

pub mod tools;

#[cfg(test)]
mod mod_tests;

/// NES Error.
#[derive(Debug)]
pub enum NesError {
    Cpu(CpuError),
    Ppu(PpuError),
}

impl From<CpuError> for NesError {
    fn from(value: CpuError) -> Self {
        Self::Cpu(value)
    }
}

impl From<PpuError> for NesError {
    fn from(value: PpuError) -> Self {
        Self::Ppu(value)
    }
}

/// NES console.
pub struct Nes {
    cpu: Cpu,
    cpu_bus: Rc<RefCell<CpuBus>>,
    ppu: Rc<RefCell<Ppu>>,
    ppu_bus: Rc<RefCell<PpuBus>>,
    joypad1: Option<Rc<RefCell<Joypad>>>,
    joypad2: Option<Rc<RefCell<Joypad>>>,
}

impl Nes {
    pub fn new(joypad1: Option<Joypad>, joypad2: Option<Joypad>) -> Self {
        let mut this = Self {
            cpu: Cpu::new(),
            cpu_bus: Rc::new(RefCell::new(CpuBus::new())),
            ppu: Rc::new(RefCell::new(Ppu::new())),
            ppu_bus: Rc::new(RefCell::new(PpuBus::new())),
            joypad1: match joypad1 {
                Some(j) => Some(Rc::new(RefCell::new(j))),
                None => None,
            },
            joypad2: match joypad2 {
                Some(j) => Some(Rc::new(RefCell::new(j))),
                None => None,
            },
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
        self.cpu.set_program_counter(addr);
    }

    pub fn run<F1, F2>(
        &mut self,
        mut cpu_callback: F1,
        mut ppu_callback: F2,
    ) -> Result<(), NesError>
    where
        F1: FnMut(&mut Cpu),
        F2: FnMut(&Frame, Option<&Rc<RefCell<Joypad>>>, Option<&Rc<RefCell<Joypad>>>) -> bool,
    {
        let mut cont = true;

        while cont {
            if self.ppu_bus.borrow_mut().cartridge_connected() {
                let nmi_before = self.ppu.borrow_mut().nmi_interrupt();

                // CPU callback is called only on instruction change
                // Not for every cycle
                if self.cpu.instruction_changed() {
                    cpu_callback(&mut self.cpu);
                }
                cont = self.cpu.tick()?;

                // PPU runs 3x faster than CPU
                for _ in 0..3 {
                    self.ppu.borrow_mut().tick()?;
                }

                let nmi_after = self.ppu.borrow_mut().nmi_interrupt();

                if !nmi_before && nmi_after {
                    cont = ppu_callback(
                        &self.ppu.borrow_mut().frame().borrow(),
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
            } else {
                cont = ppu_callback(
                    &self.ppu.borrow_mut().frame().borrow(),
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
