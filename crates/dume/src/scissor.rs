use glam::{Affine2, UVec2, Vec2};

use crate::Rect;

/// Restricts the rendering space.
///
/// A scissor is an optionally-rounded rectangle.
///
/// Note that endpoints will be rounded to the nearest
/// integer physical pixel.
///
/// Note that scissors are _not_ affected by canvas transforms
/// after they are assigned to a canvas.
#[derive(Copy, Clone, Debug)]
pub struct Scissor {
    pub region: Rect,
    pub border_radius: f32,
}

impl Scissor {
    pub fn transform(&mut self, transform: Affine2) {
        self.region = self.region.transformed(transform);
        self.border_radius = transform
            .transform_vector2(Vec2::splat(self.border_radius))
            .x;
    }
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C)]
pub struct PackedScissor {
    pub pos: UVec2,
    pub size: UVec2,
    pub border_radius: f32,
    pub _padding: f32,
}

impl From<Scissor> for PackedScissor {
    fn from(s: Scissor) -> Self {
        Self {
            pos: s.region.pos.floor().as_uvec2(),
            size: s.region.size.ceil().as_uvec2(),
            border_radius: s.border_radius,
            _padding: 0.,
        }
    }
}
