use glam::{uvec2, vec2, UVec2, Vec2};

use crate::Context;

/// A layer of rendered pixels.
///
/// You can draw to a layer through [`Canvas::render_to_layer`].
pub struct Layer {
    context: Context,
    texture: wgpu::TextureView,
    sample_texture: wgpu::TextureView,
    desc: wgpu::TextureDescriptor<'static>,
    scale_factor: f32,
}

impl Layer {
    pub(crate) fn new(context: Context, logical_size: Vec2, scale_factor: f32) -> Self {
        let physical_size = wgpu::Extent3d {
            width: (logical_size.x * scale_factor).ceil() as u32,
            height: (logical_size.y * scale_factor).ceil() as u32,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: physical_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };
        let texture = context
            .device()
            .create_texture(&desc)
            .create_view(&Default::default());
        let sample_texture = context
            .device()
            .create_texture(&wgpu::TextureDescriptor {
                sample_count: crate::SAMPLE_COUNT,
                ..desc
            })
            .create_view(&Default::default());
        Self {
            context,
            texture,
            sample_texture,
            desc,
            scale_factor,
        }
    }

    /// Gets the layer's logical size.
    pub fn size(&self) -> Vec2 {
        vec2(
            self.desc.size.width as f32 / self.scale_factor,
            self.desc.size.height as f32 / self.scale_factor,
        )
    }

    /// Gets the layer's size in physical pixels.
    pub fn physical_size(&self) -> UVec2 {
        uvec2(self.desc.size.width, self.desc.size.height)
    }

    /// Gets the HiDPI factor.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub(crate) fn texture(&self) -> &wgpu::TextureView {
        &self.texture
    }

    pub(crate) fn sample_texture(&self) -> &wgpu::TextureView {
        &self.sample_texture
    }
}
