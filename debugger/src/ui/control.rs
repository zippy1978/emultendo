use imgui::Condition;

use crate::{renderable::Renderable, emulator::state::EmulatorState};

pub struct ControlWindow {
    start_pos: [f32; 2],
}

impl ControlWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self { start_pos: [x, y] }
    }
}

impl Renderable for ControlWindow {
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
            });
    }
}
