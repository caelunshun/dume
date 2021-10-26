use glam::{vec4, Mat4, Vec2};

use crate::{renderer::Renderer, text::layout::GlyphCharacter, Context, TextBlob, TextureId};

/// A 2D canvas using `wgpu`. Modeled after the HTML5 canvas
/// API.
pub struct Canvas {
    context: Context,
    renderer: Renderer,
    target_logical_size: Vec2,
    scale_factor: f32,
}

impl Canvas {
    pub(crate) fn new(context: Context, target_logical_size: Vec2, scale_factor: f32) -> Self {
        Self {
            renderer: Renderer::new(context.device(), target_logical_size),
            context,
            target_logical_size,
            scale_factor,
        }
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
        self.renderer
            .draw_sprite(&self.context, texture, pos, width);
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
                GlyphCharacter::Glyph(glyph_id, size) => self.renderer.draw_glyph(
                    &self.context,
                    self.scale_factor,
                    *glyph_id,
                    pos + glyph.pos + glyph.offset,
                    *size,
                    glyph.font,
                    vec4(color.red, color.green, color.blue, color.alpha * alpha),
                ),
                GlyphCharacter::LineBreak => {}
                _ => todo!(),
            }
        }
        self
    }

    /// Renders a frame, flushing all current draw commands.
    ///
    /// You need to submit the provided `CommandEncoder` to a `Queue`
    /// for rendering to work.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_texture: &wgpu::TextureView,
        target_sample_texture: &wgpu::TextureView,
    ) {
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
        self.renderer
            .render(encoder, &prepared, target_texture, target_sample_texture);
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
