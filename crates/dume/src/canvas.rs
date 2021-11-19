use std::{f32::consts::TAU, iter, mem};

use glam::{vec2, vec4, Affine2, Mat4, Vec2, Vec4};
use palette::Srgba;

use crate::{
    path::{Path, PathSegment, TesselateKind},
    renderer::{Paint, Renderer},
    text::layout::GlyphCharacter,
    Context, TextBlob, TextureId,
};

/// A 2D canvas using `wgpu`. Modeled after the HTML5 canvas
/// API.
pub struct Canvas {
    context: Context,
    renderer: Renderer,
    target_logical_size: Vec2,
    scale_factor: f32,

    current_paint: Paint,
    current_path: Path,
    stroke_width: f32,

    current_transform: Affine2,
}

/// Painting
impl Canvas {
    pub(crate) fn new(context: Context, target_logical_size: Vec2, scale_factor: f32) -> Self {
        Self {
            renderer: Renderer::new(context.device(), target_logical_size),
            context,
            target_logical_size,
            scale_factor,

            current_paint: Paint::SolidColor(Vec4::ONE),
            current_path: Path::default(),
            stroke_width: 1.,

            current_transform: Affine2::IDENTITY,
        }
    }

    /// Gets the size of the drawing region.
    pub fn size(&self) -> Vec2 {
        self.target_logical_size
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
        self.renderer.draw_sprite(
            &self.context,
            self.current_transform,
            texture,
            pos,
            width,
        );
        self
    }

    /// Draws a blob of text.
    ///
    /// `pos` is the position of the top-left corner of the text.
    ///
    /// `alpha` is a multiplier applied to the alpha of each text section.
    pub fn draw_text(&mut self, text: &TextBlob, pos: Vec2, alpha: f32) -> &mut Self {
        for glyph in text.glyphs() {
            let color = glyph.color.into_format::<f32, f32>().into_linear();
            match &glyph.c {
                GlyphCharacter::Glyph(glyph_id, size, _) => {
                    self.renderer.draw_glyph(
                        &self.context,
                        self.current_transform,
                        self.scale_factor,
                        *glyph_id,
                        pos + glyph.pos + glyph.offset,
                        *size,
                        glyph.font,
                        vec4(color.red, color.green, color.blue, color.alpha * alpha),
                    );
                }
                GlyphCharacter::LineBreak => {}
                GlyphCharacter::Icon(texture_id, size) => {
                    self.draw_sprite(*texture_id, glyph.pos, *size);
                }
            }
        }
        self
    }

    /// Sets the current paint color to a solid color.
    pub fn solid_color(&mut self, color: impl Into<Srgba<u8>>) -> &mut Self {
        self.current_paint = Paint::SolidColor(srgba_to_vec4(color));
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
        self.current_paint = Paint::LinearGradient {
            p_a: self.current_transform.transform_point2(point_a),
            p_b: self.current_transform.transform_point2(point_b),
            c_a: srgba_to_vec4(color_a),
            c_b: srgba_to_vec4(color_b),
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
        self.current_paint = Paint::RadialGradient {
            center: self.current_transform.transform_point2(center),
            radius,
            c_center: srgba_to_vec4(center_color),
            c_outer: srgba_to_vec4(edge_color),
        };
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

    /// Appends a rectangle to the current path.
    pub fn rect(&mut self, pos: Vec2, size: Vec2) -> &mut Self {
        self.move_to(pos)
            .line_to(pos + vec2(size.x, 0.0))
            .line_to(pos + size)
            .line_to(pos + vec2(0.0, size.y))
            .line_to(pos)
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
        self
    }
}

/// Rendering functions
impl Canvas {
    /// Renders a frame, flushing all current draw commands.
    pub fn render(
        &mut self,
        target_texture: &wgpu::TextureView,
        target_sample_texture: &wgpu::TextureView,
    ) {
        let mut encoder = self
            .context
            .device()
            .create_command_encoder(&Default::default());

        let projection_matrix = Mat4::orthographic_lh(
            0.,
            self.target_logical_size.x,
            self.target_logical_size.y,
            0.,
            0.,
            1.,
        );
        let prepared =
            self.renderer
                .prepare_render(&self.context, self.context.device(), projection_matrix);
        self.renderer.render(
            &mut encoder,
            &prepared,
            target_texture,
            target_sample_texture,
        );

        self.reset_transform();

        self.context.queue().submit(iter::once(encoder.finish()));
    }

    /// Updates the target size of the canvas in logical pixels.
    ///
    /// If the canvas is used to draw to a window, call this whenever
    /// the window is resized.
    pub fn resize(&mut self, new_logical_size: Vec2, hidpi_factor: f32) {
        self.target_logical_size = new_logical_size;
        self.scale_factor = hidpi_factor;
        self.renderer.resize(new_logical_size);
    }

    /// Gets the context associated with this canvas.
    pub fn context(&self) -> &Context {
        &self.context
    }
}
