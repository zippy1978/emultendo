use std::sync::{Arc, RwLock};

use imgui::{Condition, Textures, Ui};
use imgui_glium_renderer::Texture;
use native_dialog::FileDialog;

use crate::{emulator::state::EmulatorState, renderable::Renderable};

pub struct CartridgeWindow {
    start_pos: [f32; 2],
}

impl CartridgeWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self { start_pos: [x, y] }
    }
}

impl Renderable for CartridgeWindow {
    fn render(
        &self,
        ui: &Ui,
        _textures: &Textures<Texture>,
        state: &mut Arc<RwLock<EmulatorState>>,
    ) {
        ui.window("Cartridge")
            .resizable(false)
            .position(self.start_pos, Condition::FirstUseEver)
            .build(|| {
                let mut state_lock = state.write().unwrap();

                match &state_lock.cartridge {
                    Some(c) => ui.text_wrapped(&c.filename),
                    None => ui.text_wrapped("No cartridge. Load one from a file."),
                };

                if ui.button("Load") {
                    let path = FileDialog::new()
                        //.set_location("~/Desktop")
                        .add_filter("iNES 1.0 Game", &["nes"])
                        .show_open_single_file()
                        .unwrap();

                    if let Some(path) = path {
                        state_lock.change_cartridge(path);
                    }
                }
            });
    }
}
