use std::{
    rc::Rc,
    sync::{Arc, RwLock},
    thread, time::Duration,
};

use self::state::{CpuState, EmulatorState, PpuState};
use emultendo_core::{
    cartridge::Cartridge,
    controller::{Joypad, JoypadButton},
    nes::Nes,
};

pub mod state;

/// Starts NES emulator in its own thread
pub fn start_emulator(state: &Arc<RwLock<EmulatorState>>) {
    let state = state.clone();

    // Run emulator in dedicated thread
    thread::spawn(move || {
        // Create console
        // plug only joypad1, otherwise Super Mario does not work
        let mut nes = Nes::new(Some(Joypad::new()), None);

        // Initial cartridge insertion detection
        while state.read().unwrap().cartridge.is_none() {
            if state.read().unwrap().cartridge.is_some() {
                state.write().unwrap().reset = true;
            }
        }

        // Run
        loop {
            nes.set_cpu_mhz(state.read().unwrap().cpu_mhz);
            let initial_cpu_mhz = nes.cpu_mhz();
            let initial_cartridge_state = Rc::new(state.read().unwrap().cartridge.clone());

            // Paused
            while state.read().unwrap().paused {
                thread::sleep(Duration::from_millis(10));
            }

            if let Some(cartridge_state) = initial_cartridge_state.as_ref() {
                // Handle reset
                if state.read().unwrap().reset {
                    nes.insert(Cartridge::from_file(&cartridge_state.filename).unwrap());
                    nes.reset();
                    state.write().unwrap().reset = false;
                }

                nes.run(
                    |cpu| {
                        let mut state_lock = state.write().unwrap();

                        // Update CPU state
                        state_lock.cpu = CpuState::from_cpu(cpu);

                        // If CPU Mhz changed: restart run loop
                        if state_lock.cpu_mhz != initial_cpu_mhz {
                            return false;
                        }

                        // Reset requested
                        if state_lock.reset {
                            return false;
                        }

                        // Pause requested
                        if state_lock.paused {
                            return  false;
                        }

                        true
                    },
                    |ppu, joypad1, _joypad2| {
                        let mut state_lock = state.write().unwrap();

                        // Update frame in state
                        state_lock.ppu = PpuState::from_ppu(ppu);

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
                            joypad1.borrow_mut().set_button_pressed_status(
                                JoypadButton::DOWN,
                                state_lock.joypad1.down,
                            );
                            joypad1.borrow_mut().set_button_pressed_status(
                                JoypadButton::LEFT,
                                state_lock.joypad1.left,
                            );
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
        }
    });
}
