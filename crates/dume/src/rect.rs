use glam::{Affine2, Vec2};

/// A rectangle.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Rect {
    /// The position of the top-left corner
    /// of this rectangle.
    pub pos: Vec2,
    /// The side lengths of this rectangle.
    pub size: Vec2,
}

impl Rect {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self { pos, size }
    }

    pub fn offset(self, offset: Vec2) -> Self {
        Self {
            pos: self.pos + offset,
            size: self.size,
        }
    }

    pub fn infinity() -> Self {
        Self {
            pos: Vec2::splat(0.0),
            size: Vec2::splat(f32::INFINITY),
        }
    }

    pub fn contains(self, pos: Vec2) -> bool {
        pos.x >= self.pos.x
            && pos.y >= self.pos.y
            && pos.x < (self.pos.x + self.size.x)
            && pos.y < (self.pos.y + self.size.y)
    }

    pub fn transformed(self, transform: Affine2) -> Self {
        Self {
            pos: transform.transform_point2(self.pos),
            size: transform.transform_vector2(self.size),
        }
    }
}
