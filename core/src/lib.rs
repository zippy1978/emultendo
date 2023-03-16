#[macro_use]
extern crate lazy_static;

pub mod cpu;
pub mod ppu;
pub mod bus;
pub mod nes;
pub mod memory;
pub mod cartridge;
pub mod renderer;
pub mod controller;


#[cfg(test)]
mod cartridge_tests;
#[cfg(test)]
mod controller_tests;

