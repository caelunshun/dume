use std::iter;

use glam::{uvec2, UVec2};

use crate::Context;

/// A layer of rendered pixels.
///
/// You can draw to a layer through [`Canvas::render_to_layer`].
pub struct Layer {
    context: Context,
    texture: wgpu::TextureView,
    desc: wgpu::TextureDescriptor<'static>,
}

impl Layer {
    pub(crate) fn new(context: Context, physical_size: UVec2, label: Option<&'static str>) -> Self {
        let physical_size = wgpu::Extent3d {
            width: physical_size.x,
            height: physical_size.y,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label,
            size: physical_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::INTERMEDIATE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        };
        let texture = context
            .device()
            .create_texture(&desc)
            .create_view(&Default::default());

        Self {
            context,
            texture,
            desc,
        }
    }

    /// Gets the layer's physical size.
    pub fn physical_size(&self) -> UVec2 {
        uvec2(self.desc.size.width, self.desc.size.height)
    }

    /// Blits the layer onto a target surface.
    ///
    /// The given texture must be of format `TARGET_FORMAT`
    /// and have `TextureUsages::RENDER_ATTACHMENT`.
    pub fn blit_onto(&self, target: &wgpu::TextureView) {
        let prepared_blit = self.context.renderer().prepare_blit(
            &self.context,
            &self.texture,
            self.physical_size(),
        );
        let mut encoder = self
            .context
            .device()
            .create_command_encoder(&Default::default());
        self.context
            .renderer()
            .blit(&mut encoder, prepared_blit, target, None);
        self.context.queue().submit(iter::once(encoder.finish()));
    }

    pub(crate) fn texture(&self) -> &wgpu::TextureView {
        &self.texture
    }
}
