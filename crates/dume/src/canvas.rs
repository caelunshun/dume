use std::{iter, mem};

use glam::{uvec2, vec2, UVec2, Vec2};
use palette::Srgba;
use swash::GlyphId;

use crate::{
    glyph::Glyph,
    renderer::{Batch, LineSegment, Node, PaintType, Shape, StrokeCap},
    text::layout::GlyphCharacter,
    Context, FontId, Rect, TextBlob, INTERMEDIATE_FORMAT,
};

/// A 2D canvas using `wgpu`. Modeled after the HTML5 canvas
/// API.
pub struct Canvas {
    context: Context,
    batch: Batch,

    current_paint: PaintType,
    current_path: Vec<LineSegment>,
    stroke_width: f32,
    stroke_cap: StrokeCap,

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
            stroke_width: 1.,
            stroke_cap: StrokeCap::Round,
            next_path_id: 0,
        }
    }

    /// Gets the size of the drawing region.
    pub fn size(&self) -> Vec2 {
        self.batch.logical_size()
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

    pub fn begin_path(&mut self) -> &mut Self {
        self.current_path.clear();
        self
    }

    pub fn move_to(&mut self, pos: Vec2) -> &mut Self {
        self.current_path.push(LineSegment {
            start: pos,
            end: pos,
        });
        self
    }

    fn last_pos(&self) -> Vec2 {
        self.current_path
            .last()
            .expect("no segments in path; call move_to first")
            .end
    }

    pub fn line_to(&mut self, pos: Vec2) -> &mut Self {
        let last = self.last_pos();
        self.current_path.push(LineSegment {
            start: last,
            end: pos,
        });
        self
    }

    /// Appends a rectangle to the current path.
    pub fn rect(&mut self, pos: Vec2, size: Vec2) -> &mut Self {
        self.move_to(pos)
            .line_to(pos + vec2(size.x, 0.0))
            .line_to(pos + size)
            .line_to(pos + vec2(0.0, size.y))
            .line_to(pos)
    }

    pub fn stroke_width(&mut self, width: f32) -> &mut Self {
        self.stroke_width = width;
        self
    }

    pub fn stroke_cap(&mut self, cap: StrokeCap) -> &mut Self {
        self.stroke_cap = cap;
        self
    }

    pub fn stroke(&mut self) -> &mut Self {
        for &segment in &self.current_path {
            self.batch.draw_node(Node {
                shape: Shape::Stroke {
                    segment,
                    width: self.stroke_width,
                    cap: self.stroke_cap,
                    path_id: self.next_path_id,
                },
                paint_type: self.current_paint,
            });
        }
        self.next_path_id += 1;
        self
    }

    pub fn fill_rect(&mut self, pos: Vec2, size: Vec2) -> &mut Self {
        self.batch.draw_node(Node {
            shape: Shape::Rect(Rect { pos, size }),
            paint_type: self.current_paint,
        });
        self
    }

    pub fn fill_circle(&mut self, center: Vec2, radius: f32) -> &mut Self {
        self.batch.draw_node(Node {
            shape: Shape::Circle { center, radius },
            paint_type: self.current_paint,
        });
        self
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
                GlyphCharacter::Icon(_, _) => todo!(),
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
        let mut glyphs = self.context.glyph_cache();
        let glyph = glyphs.glyph_or_rasterize(&self.context, font, glyph_id, size, pos);

        let (key, placement) = match glyph {
            Glyph::Empty => return,
            Glyph::InAtlas(k, p) => (k, p),
        };
        let pos = (pos + vec2(placement.left as f32, -placement.top as f32)).floor();

        let entry = glyphs.atlas().get(key);

        self.batch.draw_node(Node {
            paint_type: PaintType::Glyph {
                offset_in_atlas: entry.pos,
                origin: pos.as_u32(),
                color,
            },
            shape: Shape::Rect(Rect {
                pos,
                size: uvec2(placement.width, placement.height).as_f32(),
            }),
        });
    }

    /*
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

        /// Draws a sprite, rotating the texture by the given amount.
        pub fn draw_sprite_with_rotation(
            &mut self,
            texture: TextureId,
            pos: Vec2,
            width: f32,
            rotation: SpriteRotate,
        ) -> &mut Self {
            self.renderer.draw_sprite(
                &self.context,
                self.current_transform,
                self.current_transform_scale,
                texture,
                pos,
                width,
                rotation,
            );
            self
        }

        /// Draws a YUV texture.
        pub fn draw_yuv_texture(
            &mut self,
            texture: &YuvTexture,
            pos: Vec2,
            width: f32,
            alpha: f32,
        ) -> &mut Self {
            self.renderer
                .draw_yuv_texture(self.current_transform, texture, pos, width, alpha);
            self
        }

        /// Sets the current paint color to a solid color.
        pub fn solid_color(&mut self, color: impl Into<Srgba<u8>>) -> &mut Self {
            self.current_paint = Paint::SolidColor(srgba_to_vec4(color));
            self
        }

        /// Fills the current path with the current paint.
        pub fn fill(&mut self) -> &mut Self {
            let path = mem::take(&mut self.current_path);
            let path = (path, TesselateKind::Fill);
            self.renderer.draw_path(
                &self.context,
                self.current_transform,
                &path,
                self.current_paint,
            );
            self.current_path = path.0;
            self
        }

        /// Sets the current stroke width for path rendering.
        pub fn stroke_width(&mut self, width: f32) -> &mut Self {
            self.stroke_width = width;
            self
        }

        /// Strokes the current path with the current paint and stroke width.
        pub fn stroke(&mut self) -> &mut Self {
            let path = mem::take(&mut self.current_path);
            let path = (
                path,
                TesselateKind::Stroke {
                    width: (self.stroke_width * 100.).round() as u32,
                },
            );
            self.renderer.draw_path(
                &self.context,
                self.current_transform,
                &path,
                self.current_paint,
            );
            self.current_path = path.0;
            self
        }
    }

    fn srgba_to_vec4(srgba: impl Into<Srgba<u8>>) -> Vec4 {
        let linear = srgba.into().into_format::<f32, f32>().into_linear();
        vec4(linear.red, linear.green, linear.blue, linear.alpha)
    }

    /// Path rendering
    impl Canvas {
        /// Clears any currently staged path.
        pub fn begin_path(&mut self) -> &mut Self {
            self.current_path.segments.clear();
            self
        }

        /// Moves the cursor to the given position.
        pub fn move_to(&mut self, pos: Vec2) -> &mut Self {
            self.current_path.segments.push(PathSegment::MoveTo(pos));
            self
        }

        /// Appends a line from the current cursor to `pos`.
        pub fn line_to(&mut self, pos: Vec2) -> &mut Self {
            self.current_path.segments.push(PathSegment::LineTo(pos));
            self
        }

        /// Appends a quadratic Bezier curve through `control` to `pos`.
        pub fn quad_to(&mut self, control: Vec2, pos: Vec2) -> &mut Self {
            self.current_path
                .segments
                .push(PathSegment::QuadTo(control, pos));
            self
        }

        /// Appends a cubic Bezier curve.
        pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, pos: Vec2) -> &mut Self {
            self.current_path
                .segments
                .push(PathSegment::CubicTo(control1, control2, pos));
            self
        }

        /// Appends an arc to the current path.
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

        /// Appends a rounded rectangle to the current path.
        pub fn rounded_rect(&mut self, pos: Vec2, size: Vec2, radius: f32) -> &mut Self {
            let offset_x = vec2(radius, 0.0);
            let offset_y = vec2(0.0, radius);

            let size_x = vec2(size.x, 0.0);
            let size_y = vec2(0.0, size.y);

            self.move_to(pos + offset_x)
                .line_to(pos + size_x - offset_x)
                .quad_to(pos + size_x, pos + size_x + offset_y)
                .line_to(pos + size - offset_y)
                .quad_to(pos + size, pos + size - offset_x)
                .line_to(pos + size_y + offset_x)
                .quad_to(pos + size_y, pos + size_y - offset_y)
                .line_to(pos + offset_y)
                .quad_to(pos, pos + offset_x)
        }

        /// Appends a circle to the current path.
        pub fn circle(&mut self, center: Vec2, radius: f32) -> &mut Self {
            self.arc(center, radius, 0., TAU)
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

        /// Rotates the canvas (in radians).
        pub fn rotate(&mut self, theta: f32) -> &mut Self {
            self.current_transform = self.current_transform * Affine2::from_angle(theta);
            self
        }
        */
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
        let prepared_blit = self
            .context
            .renderer()
            .prepare_blit(&self.context, &intermediate_texture);

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
