use std::{iter, mem, sync::Arc};

use fontdb::Database;
use glam::{vec2, Affine2, Mat4, Vec2};
use palette::Srgba;

use crate::{
    glyph::GlyphKey,
    path::{Path, PathSegment, TesselateKind},
    rect::Rect,
    renderer::Renderer,
    text::layout::GlyphCharacter,
    Paragraph, SpriteId, Text, TextLayout,
};

#[derive(Debug, Copy, Clone)]
pub enum Paint {
    Solid(Srgba<u8>),
    LinearGradient {
        color_a: Srgba<u8>,
        color_b: Srgba<u8>,
        point_a: Vec2,
        point_b: Vec2,
    },
}

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

    pub(crate) renderer: Renderer,

    fonts: Database,

    current_path: Path,
    stroke_width: f32,
    paint: Paint,
}

impl Canvas {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            renderer: Renderer::new(Arc::clone(&device), Arc::clone(&queue)),
            fonts: Database::new(),

            device,
            queue,

            current_path: Path::default(),
            stroke_width: 1.0,
            paint: Paint::Solid(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX)),
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

    pub fn sprite_by_name(&self, name: &str) -> Option<SpriteId> {
        self.renderer.sprites().sprite_by_name(name)
    }

    pub fn draw_sprite(&mut self, sprite: SpriteId, pos: Vec2, width: f32) -> &mut Self {
        self.renderer.record_sprite(sprite, pos, width);
        self
    }

    pub fn create_paragraph(&self, text: Text, layout: TextLayout) -> Paragraph {
        Paragraph::new(text, layout, &self.fonts, self.renderer.sprites())
    }

    pub fn resize_paragraph(&self, paragraph: &mut Paragraph, new_max_dimensions: Vec2) {
        paragraph.update_max_dimensions(&self.fonts, new_max_dimensions);
    }

    pub fn draw_paragraph(&mut self, pos: Vec2, paragraph: &Paragraph) -> &mut Self {
        for glyph in paragraph.glyphs() {
            if !glyph.visible {
                continue;
            }
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
                    self.draw_sprite(
                        sprite,
                        glyph.pos + pos - vec2(0., glyph.bbox.size.y),
                        glyph.bbox.size.x,
                    );
                }
            }
        }

        self
    }

    pub fn begin_path(&mut self) -> &mut Self {
        self.current_path.segments.clear();
        self
    }

    pub fn move_to(&mut self, pos: Vec2) -> &mut Self {
        self.current_path.segments.push(PathSegment::MoveTo(pos));
        self
    }

    pub fn line_to(&mut self, pos: Vec2) -> &mut Self {
        self.current_path.segments.push(PathSegment::LineTo(pos));
        self
    }

    pub fn quad_to(&mut self, control: Vec2, pos: Vec2) -> &mut Self {
        self.current_path
            .segments
            .push(PathSegment::QuadTo(control, pos));
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, pos: Vec2) -> &mut Self {
        self.current_path
            .segments
            .push(PathSegment::CubicTo(control1, control2, pos));
        self
    }

    pub fn arc(
        &mut self,
        center: Vec2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> &mut Self {
        self.current_path
            .segments
            .push(PathSegment::Arc(center, radius, start_angle, end_angle));
        self
    }

    pub fn stroke_width(&mut self, width: f32) -> &mut Self {
        self.stroke_width = width;
        self
    }

    pub fn solid_color(&mut self, color: Srgba<u8>) -> &mut Self {
        self.paint = Paint::Solid(color);
        self
    }

    pub fn linear_gradient(
        &mut self,
        mut point_a: Vec2,
        mut point_b: Vec2,
        color_a: Srgba<u8>,
        color_b: Srgba<u8>,
    ) -> &mut Self {
        point_a = self.renderer.transform.transform_point2(point_a);
        point_b = self.renderer.transform.transform_point2(point_b);
        self.paint = Paint::LinearGradient {
            color_a,
            color_b,
            point_a,
            point_b,
        };
        self
    }

    pub fn stroke(&mut self) {
        let path = mem::take(&mut self.current_path);
        let kind = TesselateKind::Stroke {
            width: (self.stroke_width * 100.) as u32,
        };
        let path = (path, kind);
        self.renderer.record_path(&path, self.paint);
        self.current_path = path.0;
    }

    pub fn fill(&mut self) {
        let path = mem::take(&mut self.current_path);
        let kind = TesselateKind::Fill;
        let path = (path, kind);
        self.renderer.record_path(&path, self.paint);
        self.current_path = path.0;
    }

    pub fn scissor_rect(&mut self, mut rect: Rect) -> &mut Self {
        rect.pos = self.renderer.transform.transform_point2(rect.pos);
        rect.size = self.renderer.transform.transform_vector2(rect.size);
        self.renderer.set_scissor(rect);
        self
    }

    pub fn clear_scissor(&mut self) -> &mut Self {
        self.renderer.clear_scissor();
        self
    }

    pub fn translate(&mut self, vector: Vec2) {
        self.renderer.transform.translation += vector;
    }

    pub fn scale(&mut self, scale: f32) {
        self.renderer.transform =
            self.renderer.transform * Affine2::from_scale(glam::vec2(scale, scale));
        self.renderer.scale = scale;
    }

    pub fn reset_transform(&mut self) {
        self.renderer.transform = Affine2::IDENTITY;
        self.renderer.scale = 1.0;
    }

    pub fn render(
        &mut self,
        sampled_view: &wgpu::TextureView,
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
                    view: sampled_view,
                    resolve_target: Some(target_view),
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
