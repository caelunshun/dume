use std::{iter, mem};

use glam::{uvec2, vec2, Affine2, UVec2, Vec2};
use kurbo::{PathEl, Point};
use palette::Srgba;
use swash::GlyphId;

use crate::{
    glyph::Glyph,
    renderer::{Batch, LineSegment, Node, PaintType, Shape, StrokeCap},
    text::layout::GlyphCharacter,
    Context, FontId, Rect, Scissor, SpriteRotate, TextBlob, TextureId, INTERMEDIATE_FORMAT,
};

/// The current shape being drawn in a `Canvas`.
#[derive(Debug, Copy, Clone)]
enum PathType {
    Rect {
        rect: Rect,
        border_radius: f32,
    },
    Circle {
        center: Vec2,
        radius: f32,
    },
    /// A path of segments stored in `canvas.current_path`
    Path,
}

/// A 2D canvas using `wgpu`. Modeled after the HTML5 canvas
/// API.
pub struct Canvas {
    context: Context,
    batch: Batch,

    current_paint: PaintType,
    current_path: Vec<PathEl>,
    current_path_type: PathType,
    stroke_width: f32,
    stroke_cap: StrokeCap,
    current_transform: Affine2,
    current_transform_scale: f32,
    scissor: Option<Scissor>,

    segment_buffer: Vec<LineSegment>,

    next_path_id: u32,
}

/// Painting
impl Canvas {
    pub(crate) fn new(context: Context, target_physical_size: UVec2, scale_factor: f32) -> Self {
        let batch = context
            .renderer()
            .create_batch(target_physical_size, scale_factor);
        Self {
            context,
            batch,
            current_paint: PaintType::Solid(Srgba::default()),
            current_path: Vec::new(),
            current_path_type: PathType::Path,
            stroke_width: 1.,
            stroke_cap: StrokeCap::Round,
            current_transform: Affine2::IDENTITY,
            current_transform_scale: 1.,
            scissor: None,
            next_path_id: 0,
            segment_buffer: Vec::new(),
        }
    }

    /// Gets the size of the drawing region.
    pub fn size(&self) -> Vec2 {
        self.batch.logical_size()
    }

    /// Sets the scissor region.
    pub fn scissor(&mut self, mut scissor: Scissor) -> &mut Self {
        scissor.transform(self.current_transform);
        self.scissor = Some(scissor);
        self
    }

    /// Clears the scissor region.
    pub fn clear_scissor(&mut self) -> &mut Self {
        self.scissor = None;
        self
    }

    pub fn solid_color(&mut self, color: impl Into<Srgba<u8>>) -> &mut Self {
        self.current_paint = PaintType::Solid(color.into());
        self
    }

    /// Sets the current paint to a linear gradient between two points.
    pub fn linear_gradient(
        &mut self,
        point_a: Vec2,
        point_b: Vec2,
        color_a: impl Into<Srgba<u8>>,
        color_b: impl Into<Srgba<u8>>,
    ) -> &mut Self {
        self.current_paint = PaintType::LinearGradient {
            point_a,
            point_b,
            color_a: color_a.into(),
            color_b: color_b.into(),
        };
        self
    }

    /// Sets the current paint to a radial gradient.
    pub fn radial_gradient(
        &mut self,
        center: Vec2,
        radius: f32,
        center_color: impl Into<Srgba<u8>>,
        edge_color: impl Into<Srgba<u8>>,
    ) -> &mut Self {
        self.current_paint = PaintType::RadialGradient {
            center,
            radius,
            color_center: center_color.into(),
            color_outer: edge_color.into(),
        };
        self
    }

    fn clear_path(&mut self) {
        self.current_path.clear();
    }

    pub fn begin_path(&mut self) -> &mut Self {
        self.clear_path();
        self
    }

    pub fn move_to(&mut self, pos: Vec2) -> &mut Self {
        self.current_path_type = PathType::Path;
        self.current_path
            .push(PathEl::MoveTo(Point::new(pos.x as f64, pos.y as f64)));
        self
    }

    pub fn line_to(&mut self, pos: Vec2) -> &mut Self {
        self.current_path_type = PathType::Path;
        self.current_path
            .push(PathEl::LineTo(Point::new(pos.x as f64, pos.y as f64)));
        self
    }

    pub fn quad_to(&mut self, control: Vec2, pos: Vec2) -> &mut Self {
        self.current_path_type = PathType::Path;
        self.current_path.push(PathEl::QuadTo(
            Point::new(control.x as f64, control.y as f64),
            Point::new(pos.x as f64, pos.y as f64),
        ));
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, pos: Vec2) -> &mut Self {
        self.current_path_type = PathType::Path;
        self.current_path.push(PathEl::CurveTo(
            Point::new(control1.x as f64, control1.y as f64),
            Point::new(control2.x as f64, control2.y as f64),
            Point::new(pos.x as f64, pos.y as f64),
        ));
        self
    }

    pub fn stroke_width(&mut self, width: f32) -> &mut Self {
        self.stroke_width = width;
        self
    }

    pub fn stroke_cap(&mut self, cap: StrokeCap) -> &mut Self {
        self.stroke_cap = cap;
        self
    }

    pub fn rect(&mut self, pos: Vec2, size: Vec2) -> &mut Self {
        self.rounded_rect(pos, size, 0.)
    }

    pub fn rounded_rect(&mut self, pos: Vec2, size: Vec2, border_radius: f32) -> &mut Self {
        self.clear_path();
        let mut rect = Rect::new(pos, size);
        rect.normalize_negative_size();
        self.current_path_type = PathType::Rect {
            rect,
            border_radius,
        };
        self
    }

    pub fn circle(&mut self, center: Vec2, radius: f32) -> &mut Self {
        self.clear_path();
        self.current_path_type = PathType::Circle { center, radius };
        self
    }

    pub fn stroke(&mut self) -> &mut Self {
        if self.stroke_width * self.current_transform_scale < 0.1 {
            return self;
        }
        match self.current_path_type {
            PathType::Rect {
                rect,
                border_radius,
            } => self.stroke_rounded_rect(rect.pos, rect.size, border_radius),
            PathType::Circle { center, radius } => self.stroke_circle(center, radius),
            PathType::Path => self.stroke_path(),
        }
        self
    }

    pub fn fill(&mut self) -> &mut Self {
        match self.current_path_type {
            PathType::Rect {
                rect,
                border_radius,
            } => self.fill_rounded_rect(rect.pos, rect.size, border_radius),
            PathType::Circle { center, radius } => self.fill_circle(center, radius),
            PathType::Path => self.fill_path(),
        }
        self
    }

    fn flatten_path(&mut self) {
        self.segment_buffer.clear();
        if self.current_path.last() != Some(&PathEl::ClosePath) {
            self.current_path.push(PathEl::ClosePath);
        }

        let mut pos = Vec2::ZERO;
        kurbo::flatten(
            self.current_path.iter().copied(),
            0.25,
            |element| match element {
                PathEl::MoveTo(p) => pos = vec2(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => {
                    let target = vec2(p.x as f32, p.y as f32);
                    self.segment_buffer.push(LineSegment {
                        start: pos,
                        end: target,
                    });
                    pos = target;
                }
                PathEl::ClosePath => {}
                _ => unreachable!(),
            },
        );
    }

    fn stroke_path(&mut self) {
        self.flatten_path();
        for &segment in &self.segment_buffer {
            self.batch.draw_node(Node {
                transform: self.current_transform,
                shape: Shape::Stroke {
                    segment,
                    width: self.stroke_width,
                    cap: self.stroke_cap,
                    path_id: self.next_path_id,
                },
                paint_type: self.current_paint,
                scissor: self.scissor,
            });
        }
        self.next_path_id += 1;
    }

    fn fill_path(&mut self) {
        self.flatten_path();
        let fill_bounding_box = self.current_path_bounding_box();
        for &segment in &self.segment_buffer {
            self.batch.draw_node(Node {
                transform: self.current_transform,
                shape: Shape::Fill {
                    segment,
                    path_id: self.next_path_id,
                    fill_bounding_box,
                },
                paint_type: self.current_paint,
                scissor: self.scissor,
            });
        }
        self.next_path_id += 1;
    }

    fn current_path_bounding_box(&self) -> Rect {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(-f32::INFINITY);

        for &segment in &self.segment_buffer {
            min = min.min(segment.start).min(segment.end);
            max = max.max(segment.start).max(segment.start);
        }

        Rect {
            pos: min,
            size: max - min,
        }
    }

    fn fill_rounded_rect(&mut self, pos: Vec2, size: Vec2, border_radius: f32) {
        self.batch.draw_node(Node {
            transform: self.current_transform,
            shape: Shape::Rect {
                rect: Rect { pos, size },
                border_radius,
                stroke_width: None,
            },
            paint_type: self.current_paint,
            scissor: self.scissor,
        });
    }

    fn stroke_rounded_rect(&mut self, pos: Vec2, size: Vec2, border_radius: f32) {
        self.batch.draw_node(Node {
            transform: self.current_transform,
            shape: Shape::Rect {
                rect: Rect { pos, size },
                border_radius,
                stroke_width: Some(self.stroke_width),
            },
            paint_type: self.current_paint,
            scissor: self.scissor,
        });
    }

    fn fill_circle(&mut self, center: Vec2, radius: f32) {
        self.batch.draw_node(Node {
            transform: self.current_transform,
            shape: Shape::Circle {
                center,
                radius,
                stroke_width: None,
            },
            paint_type: self.current_paint,
            scissor: self.scissor,
        });
    }

    fn stroke_circle(&mut self, center: Vec2, radius: f32) {
        self.batch.draw_node(Node {
            transform: self.current_transform,
            shape: Shape::Circle {
                center,
                radius,
                stroke_width: Some(self.stroke_width),
            },
            paint_type: self.current_paint,
            scissor: self.scissor,
        });
    }

    /// Draws a blob of text.
    ///
    /// `pos` is the position of the top-left corner of the text.
    ///
    /// `alpha` is a multiplier applied to the alpha of each text section.
    pub fn draw_text(&mut self, text: &TextBlob, pos: Vec2, alpha: f32) -> &mut Self {
        for glyph in text.glyphs() {
            // Apply alpha multiplier
            let mut color = glyph.color;
            color.alpha = (color.alpha as f32 * alpha) as u8;

            match &glyph.c {
                GlyphCharacter::Glyph(glyph_id, size, _) => {
                    self.draw_glyph(*glyph_id, glyph.font, *size, color, pos + glyph.pos);
                }
                GlyphCharacter::LineBreak => {}
                GlyphCharacter::Icon(texture_id, size) => {
                    let dims = self.context.texture_dimensions(*texture_id);
                    let aspect_ratio = dims.y as f32 / dims.x as f32;
                    let height = aspect_ratio * *size;
                    self.draw_sprite(*texture_id, glyph.pos - vec2(0., height), *size);
                }
            }
        }
        self
    }

    fn draw_glyph(
        &mut self,
        glyph_id: GlyphId,
        font: FontId,
        size: f32,
        color: Srgba<u8>,
        pos: Vec2,
    ) {
        let scale_factor = self.batch.scale_factor();
        // Convert to physical pixels
        let size = self.current_transform_scale * size * scale_factor;
        let pos = self.current_transform.transform_point2(pos) * scale_factor;

        let mut glyphs = self.context.glyph_cache();
        let glyph = glyphs.glyph_or_rasterize(&self.context, font, glyph_id, size, pos);

        let (key, placement) = match glyph {
            Glyph::Empty => return,
            Glyph::InAtlas(k, p) => (k, p),
        };
        let pos = (pos + vec2(placement.left as f32, -placement.top as f32)).floor();

        let entry = glyphs.atlas().get(key);

        self.batch.draw_node(Node {
            transform: Affine2::IDENTITY, // transform applied manually
            paint_type: PaintType::Glyph {
                offset_in_atlas: entry.pos,
                origin: pos.as_uvec2(),
                color,
            },
            shape: Shape::Rect {
                rect: Rect {
                    pos: pos / scale_factor,
                    size: uvec2(placement.width, placement.height).as_vec2() / scale_factor,
                },
                border_radius: 0.,
                stroke_width: None,
            },
            scissor: self.scissor,
        });
    }

    /// Draws a texture / sprite on the canvas.
    ///
    /// `texture` is the ID of the texture to draw, which you
    /// may acquire through `Context::texture_for_name`.
    ///
    /// `pos` is the position in logical pixels of the top-left of the sprite.
    ///
    /// `width` is the width of the image on the canvas, also in
    /// logical pixels. The height is automatically computed from the texture's aspect ratio.
    pub fn draw_sprite(&mut self, texture: TextureId, pos: Vec2, width: f32) -> &mut Self {
        self.draw_sprite_with_rotation(texture, pos, width, SpriteRotate::Zero)
    }

    /// Draws a sprite rotated in increments of 90 degrees.
    pub fn draw_sprite_with_rotation(
        &mut self,
        texture: TextureId,
        pos: Vec2,
        width: f32,
        rotation: SpriteRotate,
    ) -> &mut Self {
        let textures = self.context.textures();
        let set_id = textures.set_for_texture(texture);
        let set = textures.texture_set(set_id);
        let texture = set.get(texture);
        let mipmap_level = texture.mipmap_level_for_target_size(
            (width * self.batch.scale_factor() * self.current_transform_scale) as u32,
        );
        let atlas_entry = set.atlas().get(*texture.mipmap_level(mipmap_level));

        let aspect_ratio = texture.size().y as f32 / texture.size().x as f32;
        let size = vec2(width, width * aspect_ratio);

        let scale = atlas_entry.size.x as f32 / width;

        let mut rect = Rect::new(pos, size);
        if matches!(rotation, SpriteRotate::One | SpriteRotate::Three) {
            rect.size = vec2(rect.size.y, rect.size.x);
        }

        let offset_in_atlas = atlas_entry.pos;
        self.batch.draw_node(Node {
            transform: self.current_transform,
            shape: Shape::Rect {
                rect,
                border_radius: 0.,
                stroke_width: None,
            },
            paint_type: PaintType::Texture {
                offset_in_atlas,
                origin: pos,
                texture_set: set_id,
                scale,
                rotation,
                texture_size: atlas_entry.size,
            },
            scissor: self.scissor,
        });

        drop(textures);

        self
    }
}

/// Canvas transformation functions
impl Canvas {
    /// Resets the current transformation to the identity matrix.
    pub fn reset_transform(&mut self) -> &mut Self {
        self.current_transform = Affine2::IDENTITY;
        self.current_transform_scale = 1.;
        self
    }

    /// Translates the canvas.
    pub fn translate(&mut self, translation: Vec2) -> &mut Self {
        self.current_transform.translation += translation;
        self
    }

    /// Scales the canvas.
    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.current_transform = self.current_transform * Affine2::from_scale(Vec2::splat(scale));
        self.current_transform_scale *= scale;
        self
    }
}

/// Rendering functions
impl Canvas {
    /// Renders a frame and then blits it onto `target_texture`.
    /// `target_texture` must have `TextureUsages::RENDER_ATTACHMENT`.
    pub fn render(&mut self, target_texture: &wgpu::TextureView) {
        let intermediate_texture = self
            .context
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("intermediate"),
                size: wgpu::Extent3d {
                    width: self.batch.physical_size().x,
                    height: self.batch.physical_size().y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: INTERMEDIATE_FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            })
            .create_view(&Default::default());

        let physical_size = self.batch.physical_size();
        let scale_factor = self.batch.scale_factor();
        let batch = mem::replace(
            &mut self.batch,
            self.context
                .renderer()
                .create_batch(physical_size, scale_factor),
        );

        // Prepare to render
        let prepared =
            self.context
                .renderer()
                .prepare_render(batch, &self.context, &intermediate_texture);
        let prepared_blit = self.context.renderer().prepare_blit(
            &self.context,
            &intermediate_texture,
            physical_size,
        );

        // Render
        let mut encoder = self
            .context
            .device()
            .create_command_encoder(&Default::default());

        self.context.renderer().render(prepared, &mut encoder);
        self.context
            .renderer()
            .blit(&mut encoder, prepared_blit, target_texture);

        self.context.queue().submit(iter::once(encoder.finish()));

        self.reset();
    }

    fn reset(&mut self) {
        self.reset_transform();
        self.current_path.clear();
        self.next_path_id = 1;
    }

    /// Updates the target size of the canvas in logical pixels.
    ///
    /// If the canvas is used to draw to a window, call this whenever
    /// the window is resized.
    pub fn resize(&mut self, new_physical_size: UVec2, hidpi_factor: f32) {
        self.batch = self
            .context
            .renderer()
            .create_batch(new_physical_size, hidpi_factor);
    }

    /// Gets the context associated with this canvas.
    pub fn context(&self) -> &Context {
        &self.context
    }
}
