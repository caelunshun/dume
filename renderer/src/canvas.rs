use std::{iter, sync::Arc};

use fontdb::Database;
use glam::{Mat4, Vec2};

use crate::{
    glyph::GlyphKey, renderer::Renderer, text::layout::GlyphCharacter, Paragraph, SpriteId, Text,
    TextLayout,
};

#[derive(Debug)]
pub struct SpriteDescriptor<'a> {
    pub name: &'a str,
    pub data: SpriteData<'a>,
}

#[derive(Debug)]
pub enum SpriteData<'a> {
    Encoded(&'a [u8]),
    Rgba {
        width: u32,
        height: u32,
        data: &'a mut [u8],
    },
}

/// A canvas for 2D rendering.
pub struct Canvas {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    renderer: Renderer,

    fonts: Database,
}

impl Canvas {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            renderer: Renderer::new(Arc::clone(&device), Arc::clone(&queue)),
            fonts: Database::new(),

            device,
            queue,
        }
    }

    pub fn load_font(&mut self, font_data: Vec<u8>) {
        self.fonts.load_font_data(font_data);
    }

    pub fn create_sprite(&mut self, descriptor: SpriteDescriptor) -> SpriteId {
        let mut image;
        let mut flat_samples;
        let (rgba_data, width, height) = match descriptor.data {
            SpriteData::Encoded(data) => {
                image = image::load_from_memory(data)
                    .expect("failed to parse image")
                    .to_rgba8();
                let width = image.width();
                let height = image.height();
                flat_samples = image.as_flat_samples_mut();
                (flat_samples.as_mut_slice(), width, height)
            }
            SpriteData::Rgba {
                width,
                height,
                data,
            } => (data, width, height),
        };

        let id = self.renderer.sprites_mut().insert(
            rgba_data,
            width,
            height,
            descriptor.name.to_owned(),
        );
        id
    }

    pub fn remove_sprite(&mut self, id: SpriteId) {
        self.renderer.sprites_mut().remove(id);
    }

    pub fn draw_sprite(&mut self, sprite: SpriteId, pos: Vec2, width: f32) -> &mut Self {
        self.renderer.record_sprite(sprite, pos, width);
        self
    }

    pub fn create_paragraph(&self, text: Text, layout: TextLayout) -> Paragraph {
        Paragraph::new(text, layout, &self.fonts, self.renderer.sprites())
    }

    pub fn draw_paragraph(&mut self, pos: Vec2, paragraph: &Paragraph) -> &mut Self {
        for glyph in paragraph.glyphs() {
            match glyph.c {
                GlyphCharacter::Char(c) => {
                    let key = GlyphKey {
                        c,
                        font: glyph.font.unwrap(),
                        size: (glyph.size * 1000.) as u64,
                    };
                    self.renderer
                        .record_glyph(key, glyph.pos + pos, glyph.color, &self.fonts);
                }
                GlyphCharacter::Icon(sprite) => {
                    self.draw_sprite(sprite, glyph.pos + pos, glyph.bbox.size.x);
                }
            }
        }

        self
    }

    pub fn render(
        &mut self,
        target_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window_size: Vec2,
    ) {
        let ortho = Mat4::orthographic_lh(0.0, window_size.x, window_size.y, 0.0, -1.0, 1.0);
        let mut prepared_sprites = self.renderer.prepare(ortho);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("doom"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.renderer.render(&mut pass, &mut prepared_sprites);
        }
    }
}
