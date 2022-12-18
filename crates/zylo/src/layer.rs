use glam::{uvec2, vec2, UVec2, Vec2};

slotmap::new_key_type! {
    /// ID of a layer.
    ///
    /// A layer corresponds to a buffer of pixels that can be drawn to.
    pub struct LayerId;
}

/// Describes the format of a layer of pixels to render to.
pub struct LayerInfo {
    physical_width: u32,
    physical_height: u32,
    hidpi_factor: f32,
}

impl LayerInfo {
    pub fn new(physical_width: u32, physical_height: u32, hidpi_factor: f32) -> Self {
        Self {
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
}
