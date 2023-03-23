use std::{borrow::Cow, error::Error, rc::Rc};

use emultendo_core::ppu::frame::Frame;
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Rect, Texture2d,
};
use imgui::{Image, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;

/// Renders NES display window.
pub struct DisplayWindow {
    texture_id: Option<TextureId>,
}

impl DisplayWindow {
    const PIXEL_SCALE: f32 = 2.0;

    pub fn new() -> Self {
        Self { texture_id: None }
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

    pub fn update(
        &mut self,
        frame: &Frame,
        textures: &Textures<Texture>,
    ) -> Result<(), Box<dyn Error>> {
        let raw = RawImage2d {
            data: Cow::Owned(frame.data().to_vec()),
            width: Frame::WIDTH as u32,
            height: Frame::HEIGHT as u32,
            format: ClientFormat::U8U8U8,
        };

        match self.texture_id {
            Some(tid) => {
                let t = textures.get(tid).unwrap();
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
            None => {
                return Err("textures not registered".into());
            }
        };

        Ok(())
    }
    pub fn render(&self, ui: &Ui) {
        ui.window("Display")
            .resizable(false)
            //.no_decoration()
            .content_size([
                Frame::WIDTH as f32 * Self::PIXEL_SCALE,
                Frame::HEIGHT as f32 * Self::PIXEL_SCALE,
            ])
            .build(|| {
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
