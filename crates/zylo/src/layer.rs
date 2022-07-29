use glam::{uvec2, vec2, UVec2, Vec2};

use crate::{backend::BackendLayer, Color, Context};

/// A layer of pixels to render to.
pub struct Layer {
    inner: Box<dyn BackendLayer>,
    physical_width: u32,
    physical_height: u32,
    hidpi_factor: f32,
}

impl Layer {
    pub(crate) fn new(
        context: &Context,
        physical_width: u32,
        physical_height: u32,
        hidpi_factor: f32,
    ) -> Self {
        Self {
            inner: context
                .backend()
                .create_layer(physical_width, physical_height, hidpi_factor),
            physical_width,
            physical_height,
            hidpi_factor,
        }
    }

    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    pub fn physical_size(&self) -> UVec2 {
        uvec2(self.physical_width(), self.physical_height())
    }

    pub fn logical_width(&self) -> f32 {
        self.physical_width() as f32 / self.hidpi_factor
    }

    pub fn logical_height(&self) -> f32 {
        self.physical_height() as f32 / self.hidpi_factor
    }

    pub fn logical_size(&self) -> Vec2 {
        vec2(self.logical_width(), self.logical_height())
    }

    pub fn hidpi_factor(&self) -> f32 {
        self.hidpi_factor
    }

    pub fn to_argb(&self) -> Vec<u32> {
        self.inner.to_argb()
    }

    pub fn fill(&mut self, color: Color) {
        self.inner.fill(color);
    }

    pub fn inner(&self) -> &dyn BackendLayer {
        &*self.inner
    }

    pub fn inner_mut(&mut self) -> &mut dyn BackendLayer {
        &mut *self.inner
    }
}
