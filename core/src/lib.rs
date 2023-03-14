#[macro_use]
extern crate lazy_static;

pub mod cpu;
pub mod bus;
pub mod nes;
pub mod memory;
pub mod cartridge;


#[cfg(test)]
mod cartridge_tests;
#[cfg(test)]
mod nes_tests;
