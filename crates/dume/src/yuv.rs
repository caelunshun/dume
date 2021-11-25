use std::{num::NonZeroU32, sync::Arc};

use glam::UVec2;

use crate::Context;

/// A YUV image consisting of three planes.
///
/// Use with [`Canvas::draw_yuv_texture`].
///
/// YUV texture formats are commonly used to render videos.
pub struct YuvTexture {
    cx: Context,

    y_texture: wgpu::Texture,
    u_texture: wgpu::Texture,
    v_texture: wgpu::Texture,

    pub(crate) y_texture_view: Arc<wgpu::TextureView>,
    pub(crate) u_texture_view: Arc<wgpu::TextureView>,
    pub(crate) v_texture_view: Arc<wgpu::TextureView>,

    pub(crate) size: UVec2,

    y_size: Size,
    u_size: Size,
    v_size: Size,
}

impl YuvTexture {
    pub(crate) fn new(cx: &Context, size: UVec2, y_size: Size, u_size: Size, v_size: Size) -> Self {
        let device = cx.device();
        let (y_texture, y_texture_view) = create_plane(device, y_size.apply(size));
        let (u_texture, u_texture_view) = create_plane(device, u_size.apply(size));
        let (v_texture, v_texture_view) = create_plane(device, v_size.apply(size));

        Self {
            cx: cx.clone(),
            y_texture,
            u_texture,
            v_texture,
            y_texture_view,
            u_texture_view,
            v_texture_view,
            size,
            y_size,
            u_size,
            v_size,
        }
    }

    /// Updates the texture contents for each plane.
    pub fn update(&self, y_plane: &[u8], u_plane: &[u8], v_plane: &[u8]) {
        self.update_plane(&self.y_texture, self.y_size, y_plane);
        self.update_plane(&self.u_texture, self.u_size, u_plane);
        self.update_plane(&self.v_texture, self.v_size, v_plane);
    }

    fn update_plane(&self, plane: &wgpu::Texture, plane_size: Size, data: &[u8]) {
        let size = plane_size.apply(self.size);
        self.cx.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: plane,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(size.x).unwrap()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn create_plane(device: &wgpu::Device, size: UVec2) -> (wgpu::Texture, Arc<wgpu::TextureView>) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    });
    let view = texture.create_view(&Default::default());
    (texture, Arc::new(view))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Size {
    Full,
    Half,
}

impl Size {
    pub fn apply(self, size: UVec2) -> UVec2 {
        match self {
            Size::Full => size,
            Size::Half => size / 2,
        }
    }
}
