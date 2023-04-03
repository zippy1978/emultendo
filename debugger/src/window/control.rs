use imgui::Condition;

use crate::{emulator::state::EmulatorState, widget::Widget};

pub struct ControlWindow {
    start_pos: [f32; 2],
}

impl ControlWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self { start_pos: [x, y] }
    }
}

impl Widget for ControlWindow {
    fn render(
        &self,
        ui: &imgui::Ui,
        _textures: &imgui::Textures<imgui_glium_renderer::Texture>,
        state: &mut std::sync::Arc<std::sync::RwLock<EmulatorState>>,
    ) {
        ui.window("Control")
            .resizable(false)
            .position(self.start_pos, Condition::FirstUseEver)
            .build(|| {
                let mut state_lock = state.write().unwrap();

                if ui.button("Reset") {
                    state_lock.reset = true;
                }

                ui.same_line();

                if ui.button(if state_lock.paused {
                    "Resume###Pause"
                } else {
                    "Pause###Pause"
                }) {
                    state_lock.paused = !state_lock.paused;
                }
            });
    }
}
