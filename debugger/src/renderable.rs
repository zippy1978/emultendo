use std::sync::{RwLock, Arc};

use imgui::{Ui, Textures};
use imgui_glium_renderer::Texture;

use crate::emulator::state::EmulatorState;

/// Defines a renderable element.
pub trait Renderable {
    fn render(&self, ui: &Ui, textures: &Textures<Texture>, state: &mut Arc<RwLock<EmulatorState>>);
}