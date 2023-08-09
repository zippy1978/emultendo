use std::sync::{RwLock, Arc};

use imgui::{Ui, Textures};
use imgui_glium_renderer::Texture;

use crate::emulator::state::EmulatorState;

/// Widget.
pub trait Widget {
    fn render(&self, ui: &Ui, textures: &Textures<Texture>, state: &mut Arc<RwLock<EmulatorState>>);
}