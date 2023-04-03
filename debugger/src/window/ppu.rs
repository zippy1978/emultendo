use std::{
    borrow::{Borrow, Cow},
    error::Error,
    rc::Rc,
    sync::{Arc, RwLock},
};

use emultendo_core::ppu::{
    frame::Frame,
    palette::{self, bg_palette},
};
use glium::{
    backend::Facade,
    texture::{ClientFormat, RawImage2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior},
    Rect, Texture2d,
};
use imgui::{Condition, Image, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;

use crate::{emulator::state::EmulatorState, widget::Widget};

pub struct PpuWindow {
    start_pos: [f32; 2],
    texture_id: Option<TextureId>,
}

impl PpuWindow {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            start_pos: [x, y],
            texture_id: None,
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
                data: Cow::Owned(vec![0 as u8; Frame::WIDTH * Frame::HEIGHT * 3 * 4]),
                width: Frame::WIDTH as u32 * 2,
                height: Frame::HEIGHT as u32 * 2,
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

    /// Renders a name table to a RawImage2d.
    fn render_name_table(&self, state: &EmulatorState, name_table: &[u8]) -> RawImage2d<u8> {
        let mut data = vec![0 as u8; Frame::WIDTH * Frame::HEIGHT * 3];

        if let Some(cartridge) = &state.borrow().cartridge {
            let bank = state.borrow().ppu.ctrl.bknd_pattern_addr;
            let attribute_table = &name_table[0x3c0..0x400];
            let chr_rom = &cartridge.chr_rom;

            for i in 0..name_table.len() {
                let tile = name_table[i] as u16;
                let tile_x = i % 32;
                let tile_y = i / 32;
                let palette = bg_palette(
                    &state.borrow().ppu.palette_table,
                    attribute_table,
                    tile_x,
                    tile_y,
                );
                let tile = &chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];

                for y in 0..=7 {
                    let mut upper = tile[y];
                    let mut lower = tile[y + 8];

                    for x in (0..=7).rev() {
                        let value = (1 & lower) << 1 | (1 & upper);
                        upper = upper >> 1;
                        lower = lower >> 1;
                        let rgb = match value {
                            0 => {
                                palette::SYSTEM_PALETTE
                                    [state.borrow().ppu.palette_table[0] as usize]
                            }
                            1 => palette::SYSTEM_PALETTE[palette[1] as usize],
                            2 => palette::SYSTEM_PALETTE[palette[2] as usize],
                            3 => palette::SYSTEM_PALETTE[palette[3] as usize],
                            _ => panic!("unreachable case"),
                        };
                        let base = (tile_y * 8 + y) * 3 * Frame::WIDTH + (tile_x * 8 + x) * 3;
                        if base + 2 < data.len() {
                            data[base] = rgb.0;
                            data[base + 1] = rgb.1;
                            data[base + 2] = rgb.2;
                        }
                    }
                }
            }
        }

        RawImage2d {
            data: Cow::Owned(data),
            width: Frame::WIDTH as u32,
            height: Frame::HEIGHT as u32,
            format: ClientFormat::U8U8U8,
        }
    }

    fn render_name_tables(&self, state: &EmulatorState) -> Vec<RawImage2d<u8>> {
        let mut result = vec![];

        if let Some(cartridge) = &state.borrow().cartridge {
            let vram = state.borrow().ppu.vram;

            match cartridge.screen_mirroring {
                emultendo_core::cartridge::Mirroring::Vertical => {
                    result.push(self.render_name_table(&state, &vram[0..0x400]));
                    result.push(self.render_name_table(&state, &vram[0x400..0x800]));
                    result.push(self.render_name_table(&state, &vram[0..0x400]));
                    result.push(self.render_name_table(&state, &vram[0x400..0x800]));
                }
                emultendo_core::cartridge::Mirroring::Horizontal => {
                    result.push(self.render_name_table(&state, &vram[0..0x400]));
                    result.push(self.render_name_table(&state, &vram[0..0x400]));
                    result.push(self.render_name_table(&state, &vram[0x400..0x800]));
                    result.push(self.render_name_table(&state, &vram[0x400..0x800]));
                }
                // Does nothing
                emultendo_core::cartridge::Mirroring::FourScreen => (),
            };
        }

        result
    }
}

impl Widget for PpuWindow {
    fn render(
        &self,
        ui: &Ui,
        textures: &Textures<Texture>,
        state: &mut Arc<RwLock<EmulatorState>>,
    ) {
        if let Some(texture_id) = self.texture_id {
            let name_table_renderings = self.render_name_tables(&state.read().unwrap().borrow());
            let t = textures.get(texture_id).unwrap();
            for (i, raw) in name_table_renderings.into_iter().enumerate() {
                t.texture.write(
                    Rect {
                        left: (i as u32 % 2) * raw.width,
                        bottom: (i as u32 / 2) * raw.height,
                        width: raw.width,
                        height: raw.height,
                    },
                    raw,
                );
            }
        }

        ui.window("PPU")
            .resizable(true)
            .position(self.start_pos, Condition::FirstUseEver)
            .build(|| {
                ui.text("Nametables");

                // Render name tables (4 screens)
                if let Some(texture_id) = self.texture_id {
                    Image::new(
                        texture_id,
                        [Frame::WIDTH as f32 * 2.0, Frame::HEIGHT as f32 * 2.0],
                    )
                    .build(ui);
                }
            });
    }
}
