use std::{borrow::Cow, error::Error, rc::Rc, sync::Arc};

use emultendo_core::ppu::frame::Frame;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Rect, Texture2d,
};
use imgui::{Condition, Image, Key, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;

use crate::{emulator::state::EmulatorState, renderable::Renderable};

/// Renders NES display window.
pub struct DisplayWindow {
    texture_id: Option<TextureId>,
    start_pos: [f32; 2],
}

impl DisplayWindow {
    const PIXEL_SCALE: f32 = 2.0;

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            texture_id: None,
            start_pos: [x, y],
        }
    }

    pub fn register_textures<F>(
        &mut self,
        gl_ctx: &F,
        textures: &mut Textures<Texture>,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Facade,
    {
        if self.texture_id.is_none() {
            let raw = RawImage2d {
                data: Cow::Owned(vec![0 as u8; Frame::WIDTH * Frame::HEIGHT * 3]),
                width: Frame::WIDTH as u32,
                height: Frame::HEIGHT as u32,
                format: ClientFormat::U8U8U8,
            };
            let gl_texture = Texture2d::new(gl_ctx, raw)?;
            let texture = Texture {
                texture: Rc::new(gl_texture),
                sampler: SamplerBehavior {
                    magnify_filter: MagnifySamplerFilter::Nearest,
                    minify_filter: MinifySamplerFilter::Nearest,
                    ..Default::default()
                },
            };
            let texture_id = textures.insert(texture);

            self.texture_id = Some(texture_id);
        }

        Ok(())
    }
}

impl Renderable for DisplayWindow {
    fn render(
        &self,
        ui: &Ui,
        textures: &Textures<Texture>,
        state: &mut Arc<std::sync::RwLock<EmulatorState>>,
    ) {
        // Update texture with frame
        let raw = RawImage2d {
            data: Cow::Owned(state.read().unwrap().frame.data().to_vec()),
            width: Frame::WIDTH as u32,
            height: Frame::HEIGHT as u32,
            format: ClientFormat::U8U8U8,
        };

        if let Some(texture_id) = self.texture_id {
            let t = textures.get(texture_id).unwrap();
            t.texture.write(
                Rect {
                    left: 0,
                    bottom: 0,
                    width: Frame::WIDTH as u32,
                    height: Frame::HEIGHT as u32,
                },
                raw,
            );
        }

        ui.window("Display")
            .resizable(false)
            .position(self.start_pos, Condition::FirstUseEver)
            .build(|| {
                let mut state_lock = state.write().unwrap();

                // Update joypad state
                state_lock.joypad1.start = ui.is_key_down(Key::Enter);
                state_lock.joypad1.select = ui.is_key_down(Key::Space);
                state_lock.joypad1.up = ui.is_key_down(Key::UpArrow);
                state_lock.joypad1.down = ui.is_key_down(Key::DownArrow);
                state_lock.joypad1.left = ui.is_key_down(Key::LeftArrow);
                state_lock.joypad1.right = ui.is_key_down(Key::RightArrow);
                state_lock.joypad1.a = ui.is_key_down(Key::A);
                state_lock.joypad1.b = ui.is_key_down(Key::S);

                // Render frame
                if let Some(texture_id) = self.texture_id {
                    Image::new(
                        texture_id,
                        [
                            (Frame::WIDTH as f32 * Self::PIXEL_SCALE) as f32,
                            (Frame::HEIGHT as f32 * Self::PIXEL_SCALE) as f32,
                        ],
                    )
                    .build(ui);
                }
            });
    }
}
