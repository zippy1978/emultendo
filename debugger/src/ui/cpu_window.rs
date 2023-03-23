use std::sync::{Arc, RwLock};

use imgui::{Textures, Ui, Condition};
use imgui_glium_renderer::Texture;

use crate::{emulator::EmulatorState, renderable::Renderable};

pub struct CpuWindow {
    start_pos: [f32; 2],
}

impl CpuWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self {start_pos: [x, y]}
    }
}

impl Renderable for CpuWindow {
    fn render(&self, ui: &Ui, _textures: &Textures<Texture>, state: &mut Arc<RwLock<EmulatorState>>) {
        ui.window("CPU")
            .resizable(false)
            .position(self.start_pos, Condition::FirstUseEver)
            .content_size([300.0, 30.0])
            .build(|| {
                let state = state.read().unwrap();

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
                    ui.text(format!("0x{:02x}", state.cpu.register_a));
                    ui.table_set_column_index(1);
                    ui.text(format!("0x{:02x}", state.cpu.register_x));
                    ui.table_set_column_index(2);
                    ui.text(format!("0x{:02x}", state.cpu.register_y));
                    ui.table_set_column_index(3);
                    ui.text(format!("0x{:02x}", state.cpu.status));
                    ui.table_set_column_index(4);
                    ui.text(format!("0x{:02x}", state.cpu.program_counter));
                    ui.table_set_column_index(5);
                    ui.text(format!("0x{:02x}", state.cpu.stack_pointer));
                }
            });
    }
}
