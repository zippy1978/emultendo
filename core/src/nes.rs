use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::{CPUError, CPU},
};

/// NES Error.
#[derive(Debug)]
pub enum NESError {
    CPU(CPUError),
}

impl From<CPUError> for NESError {
    fn from(value: CPUError) -> Self {
        Self::CPU(value)
    }
}

/// NES console.
pub struct NES {
    cpu: CPU,
    bus: Rc<RefCell<Bus>>,
}

impl NES {
    pub fn new() -> Self {
        let mut this = Self {
            cpu: CPU::new(),
            bus: Rc::new(RefCell::new(Bus::new())),
        };
        this.cpu.connect_bus(&this.bus);
        this
    }

    pub fn insert(&mut self, cartridge: Cartridge) {
        self.bus.borrow_mut().connect_cartridge(cartridge);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    #[cfg(test)]
    pub fn start_at(&mut self, addr: u16) {
        self.cpu.program_counter = addr;
    }

    pub fn run<F>(&mut self, callback: F) -> Result<(), NESError>
    where
        F: FnMut(&mut CPU),
    {
        Ok(self.cpu.run_with_callback(callback)?)
    }
}
