use std::{
    sync::{Arc, RwLock},
    thread,
};

use emultendo_core::{
    cartridge::Cartridge,
    controller::{Joypad, JoypadButton},
    cpu::{Cpu, CpuFlags},
    nes::{Nes, CPU_MHZ},
    ppu::frame::Frame,
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

    fn from_cpu(cpu: &Cpu) -> Self {
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

/// Emulator state.
pub struct EmulatorState {
    pub frame: Frame,
    pub cpu: CpuState,
    pub joypad1: JoypadState,
    pub cpu_mhz: f32,
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            frame: Frame::new(),
            cpu: CpuState::new(),
            joypad1: JoypadState::new(),
            cpu_mhz: CPU_MHZ,
        }
    }
}

/// Starts NES emulator in its own thread
pub fn start_emulator(state: &Arc<RwLock<EmulatorState>>) {
    let state = state.clone();

    // Run emulator in dedicated thread
    thread::spawn(move || {
        //let current_frame = &current_frame.clone();
        let game_filename = Some(Box::new("../games/smario.nes".to_string()));
        //let game_filename: Option<Box<String>> = None;

        // Create console
        // plug only joypad1, other Super Mario does not work
        let mut nes = Nes::new(Some(Joypad::new()), None);

        // Load game to cartridge (if game file)
        // then insert cartridge and reset
        if let Some(game_filename) = &game_filename {
            let cartridge = Cartridge::from_file(game_filename.as_ref()).unwrap();
            nes.insert(cartridge);
            nes.reset();
        }

        // Run
        loop {
            nes.set_cpu_mhz(state.read().unwrap().cpu_mhz);
            let initial_cpu_mhz = nes.cpu_mhz();
            nes.run(
                |cpu| {
                    let mut state_lock = state.write().unwrap();

                    // Update CPU state
                    state_lock.cpu = CpuState::from_cpu(cpu);

                    // If CPU Mhz changed: restart run loop
                    if state_lock.cpu_mhz != initial_cpu_mhz {
                        return false;
                    }

                    true
                },
                |frame, joypad1, _joypad2| {
                    let mut state_lock = state.write().unwrap();

                    // Update frame in state
                    state_lock.frame = frame.clone();

                    // Update Joypad from state
                    if let Some(joypad1) = &joypad1 {
                        joypad1.borrow_mut().set_button_pressed_status(
                            JoypadButton::BUTTON_A,
                            state_lock.joypad1.a,
                        );
                        joypad1.borrow_mut().set_button_pressed_status(
                            JoypadButton::BUTTON_B,
                            state_lock.joypad1.b,
                        );
                        joypad1
                            .borrow_mut()
                            .set_button_pressed_status(JoypadButton::UP, state_lock.joypad1.up);
                        joypad1
                            .borrow_mut()
                            .set_button_pressed_status(JoypadButton::DOWN, state_lock.joypad1.down);
                        joypad1
                            .borrow_mut()
                            .set_button_pressed_status(JoypadButton::LEFT, state_lock.joypad1.left);
                        joypad1.borrow_mut().set_button_pressed_status(
                            JoypadButton::RIGHT,
                            state_lock.joypad1.right,
                        );
                        joypad1.borrow_mut().set_button_pressed_status(
                            JoypadButton::START,
                            state_lock.joypad1.start,
                        );
                        joypad1.borrow_mut().set_button_pressed_status(
                            JoypadButton::SELECT,
                            state_lock.joypad1.select,
                        );
                    }

                    true
                },
            )
            .unwrap();
        }
    });
}
