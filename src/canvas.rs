use glam::{Mat4, Vec2};

use crate::{renderer::Renderer, Context, TextureId};

/// A 2D canvas using `wgpu`. Modeled after the HTML5 canvas
/// API.
pub struct Canvas {
    context: Context,
    renderer: Renderer,
    target_logical_size: Vec2,
}

impl Canvas {
    pub(crate) fn new(context: Context, target_logical_size: Vec2) -> Self {
        Self {
            renderer: Renderer::new(context.device(), target_logical_size),
            context,
            target_logical_size,
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
        self.renderer.render(
            &self.context,
            self.context.device(),
            encoder,
            &prepared,
            target_texture,
            target_sample_texture,
        );
    }
}
