use std::path::PathBuf;

use emultendo_core::{
    cartridge::{Cartridge, Mirroring},
    cpu::{Cpu, CpuFlags},
    nes::CPU_MHZ,
    ppu::{frame::Frame, Ppu},
};

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
    pub chr_rom: Vec<u8>,
    pub screen_mirroring: Mirroring,
}

impl CartridgeState {
    pub fn new(filename: &str, chr_rom: Vec<u8>, screen_mirroring: Mirroring) -> Self {
        Self {
            filename: filename.to_string(),
            chr_rom,
            screen_mirroring,
        }
    }
}

/// PPU State.
pub struct PpuState {
    pub frame: Frame,
    pub vram: [u8; 2048],
    pub palette_table: [u8; 32],
    pub ctrl: PpuControlState,
    pub scroll: PpuScrollState,
}

impl PpuState {
    pub fn new() -> Self {
        Self {
            frame: Frame::new(),
            vram: [0; 2048],
            palette_table: [0; 32],
            ctrl: PpuControlState::new(),
            scroll: PpuScrollState::new(),
        }
    }

    pub fn from_ppu(ppu: &Ppu) -> Self {
        Self {
            frame: ppu.frame().borrow_mut().clone(),
            vram: if let Some(bus) = &ppu.bus() {
                bus.as_ref().borrow().vram().clone()
            } else {
                [0; 2048]
            },
            palette_table: if let Some(bus) = &ppu.bus() {
                bus.as_ref().borrow().palette_table().clone()
            } else {
                [0; 32]
            },
            ctrl: PpuControlState {
                bknd_pattern_addr: ppu.ctrl().bknd_pattern_addr(),
            },
            scroll: PpuScrollState {
                scroll_x: ppu.scroll().scroll_x,
                scroll_y: ppu.scroll().scroll_y,
            },
        }
    }
}

/// PPU Control state
pub struct PpuControlState {
    pub bknd_pattern_addr: u16,
}

impl PpuControlState {
    pub fn new() -> Self {
        Self {
            bknd_pattern_addr: 0,
        }
    }
}

/// PPU Scroll state
pub struct PpuScrollState {
    pub scroll_x: u8,
    pub scroll_y: u8,
}

impl PpuScrollState {
    pub fn new() -> Self {
        Self {
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

/// Emulator state.
pub struct EmulatorState {
    pub ppu: PpuState,
    pub cpu: CpuState,
    pub joypad1: JoypadState,
    pub cartridge: Option<CartridgeState>,
    pub reset: bool,
    pub paused: bool,
    pub cpu_mhz: f32,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            ppu: PpuState::new(),
            cpu: CpuState::new(),
            joypad1: JoypadState::new(),
            cartridge: None,
            reset: false,
            paused: false,
            cpu_mhz: CPU_MHZ,
        }
    }

    pub fn change_cartridge(&mut self, path: PathBuf) {
        let cartridge = Cartridge::from_file(&path).unwrap();
        self.cartridge = Some(CartridgeState::new(
            &path.as_os_str().to_str().unwrap(),
            cartridge.chr_rom().clone(),
            cartridge.screen_mirroring().clone(),
        ));
        self.reset = true;
    }
}
