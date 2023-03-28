use std::path::PathBuf;

use emultendo_core::{cpu::{CpuFlags, Cpu}, ppu::frame::Frame, nes::CPU_MHZ};


/// Joypad state.
pub struct JoypadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl JoypadState {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            start: false,
            select: false,
        }
    }
}

/// CPU state.
pub struct CpuState {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub program_counter: u16,
    pub stack_pointer: u8,
}

impl CpuState {
    fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_pointer: 0,
        }
    }

    pub fn from_cpu(cpu: &Cpu) -> Self {
        Self {
            register_a: cpu.register_a(),
            register_x: cpu.register_x(),
            register_y: cpu.register_y(),
            status: cpu.status(),
            program_counter: cpu.program_counter(),
            stack_pointer: cpu.stack_pointer(),
        }
    }
}

/// Cartridge state.
#[derive(PartialEq, Eq, Clone)]
pub struct CartridgeState {
    pub filename: String,
}

impl CartridgeState {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}

/// Emulator state.
pub struct EmulatorState {
    pub frame: Frame,
    pub cpu: CpuState,
    pub joypad1: JoypadState,
    pub cartridge: Option<CartridgeState>,
    pub reset: bool,
    pub cpu_mhz: f32,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            frame: Frame::new(),
            cpu: CpuState::new(),
            joypad1: JoypadState::new(),
            cartridge: None,
            reset: false,
            cpu_mhz: CPU_MHZ,
        }
    }

    pub fn change_cartridge(&mut self, path: PathBuf) {
        self.cartridge = Some(CartridgeState::new(&path.as_os_str().to_str().unwrap()));
        self.reset = true;
    }

}
