use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, SystemTimeError},
};

use spin_sleep::LoopHelper;

use crate::{
    bus::{cpu_bus::CpuBus, ppu_bus::PpuBus},
    cartridge::Cartridge,
    controller::Joypad,
    cpu::{Cpu, CpuError},
    ppu::{frame::Frame, Ppu, PpuError},
};

pub mod tools;

#[cfg(test)]
mod mod_tests;

// Source: https://www.nesdev.org/wiki/Cycle_reference_chart
pub const CPU_MHZ: f32 = 1.789773;

/// NES Error.
#[derive(Debug)]
pub enum NesError {
    Cpu(CpuError),
    Ppu(PpuError),
    Clock(String),
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

impl From<SystemTimeError> for NesError {
    fn from(value: SystemTimeError) -> Self {
        Self::Clock(value.to_string())
    }
}

/// NES console.
pub struct Nes {
    cpu: Cpu,
    cpu_bus: Rc<RefCell<CpuBus>>,
    cpu_mhz: f32,
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
            cpu_mhz: CPU_MHZ,
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

    pub fn set_cpu_mhz(&mut self, mhz: f32) {
        self.cpu_mhz = mhz;
    }

    pub fn cpu_mhz(&self) -> f32 {
        self.cpu_mhz
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
        F1: FnMut(&mut Cpu) -> bool,
        F2: FnMut(&Frame, Option<&Rc<RefCell<Joypad>>>, Option<&Rc<RefCell<Joypad>>>) -> bool,
    {
        let mut cont = true;

        let mut loop_helper = LoopHelper::builder().build_without_target_rate();

        while cont {
            if self.ppu_bus.borrow_mut().cartridge_connected() {
                let delta = loop_helper.loop_start();

                let cycle_duration =
                    Duration::from_nanos(1_000_000_000 / (self.cpu_mhz * 1_000_000.0) as u64);

                let nmi_before = self.ppu.borrow_mut().nmi_interrupt();

                // CPU callback is called only on instruction change
                // Not for every cycle
                if self.cpu.instruction_changed() {
                    cont = cont && cpu_callback(&mut self.cpu);
                }
                cont = cont && self.cpu.tick()?;

                // PPU runs 3x faster than CPU
                for _ in 0..3 {
                    self.ppu.borrow_mut().tick()?;
                }

                let nmi_after = self.ppu.borrow_mut().nmi_interrupt();

                if !nmi_before && nmi_after {
                    // TODO: handle frame drop ?
                    cont = cont
                        && ppu_callback(
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

                // Wait for cycle duration end
                // DO NOT use a thread::sleep : not accurate enough !
                if let Some(wait) = cycle_duration.checked_sub(delta) {
                    spin_sleep::sleep(wait);
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
