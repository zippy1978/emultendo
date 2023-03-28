use std::sync::{Arc, RwLock};

use imgui::{Condition, Textures, Ui};
use imgui_glium_renderer::Texture;

use crate::{emulator::state::EmulatorState, renderable::Renderable};

pub struct CpuWindow {
    start_pos: [f32; 2],
}

impl CpuWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self { start_pos: [x, y] }
    }
}

impl Renderable for CpuWindow {
    fn render(
        &self,
        ui: &Ui,
        _textures: &Textures<Texture>,
        state: &mut Arc<RwLock<EmulatorState>>,
    ) {
        ui.window("CPU")
            .resizable(false)
            .position(self.start_pos, Condition::FirstUseEver)
            .build(|| {
                let mut state_lock = state.write().unwrap();

                ui.text("Registers");

                let num_cols = 6;

                let flags = imgui::TableFlags::ROW_BG
                    | imgui::TableFlags::BORDERS_H
                    | imgui::TableFlags::BORDERS_V;

                if let Some(_t) =
                    ui.begin_table_with_sizing("cpu_registers", num_cols, flags, [300.0, 10.0], 0.0)
                {
                    ui.table_setup_column("A");
                    ui.table_setup_column("X");
                    ui.table_setup_column("Y");
                    ui.table_setup_column("P");
                    ui.table_setup_column("PC");
                    ui.table_setup_column("SP");

                    ui.table_headers_row();
                    ui.table_next_row();

                    ui.table_set_column_index(0);
                    ui.text(format!("0x{:02x}", state_lock.cpu.register_a));
                    ui.table_set_column_index(1);
                    ui.text(format!("0x{:02x}", state_lock.cpu.register_x));
                    ui.table_set_column_index(2);
                    ui.text(format!("0x{:02x}", state_lock.cpu.register_y));
                    ui.table_set_column_index(3);
                    ui.text(format!("0x{:02x}", state_lock.cpu.status));
                    ui.table_set_column_index(4);
                    ui.text(format!("0x{:02x}", state_lock.cpu.program_counter));
                    ui.table_set_column_index(5);
                    ui.text(format!("0x{:02x}", state_lock.cpu.stack_pointer));
                }

                ui.separator();

                ui.slider_config("Clock", 0.1, 10.0)
                    .display_format("%.02fMhz")
                    .build(&mut state_lock.cpu_mhz);
            });
    }
}
