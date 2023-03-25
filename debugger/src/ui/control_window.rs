use std::sync::{Arc, RwLock};

use imgui::{Textures, Ui, Condition};
use imgui_glium_renderer::Texture;

use crate::{emulator::EmulatorState, renderable::Renderable};

pub struct ControlWindow {
    start_pos: [f32; 2],
}

impl ControlWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self {start_pos: [x, y]}
    }
}

impl Renderable for ControlWindow {
    fn render(&self, ui: &Ui, _textures: &Textures<Texture>, state: &mut Arc<RwLock<EmulatorState>>) {
        ui.window("Control")
        .resizable(false)
        .position(self.start_pos, Condition::FirstUseEver)
        .content_size([300.0, 30.0])
        .build(|| {
           
        });
    }
}