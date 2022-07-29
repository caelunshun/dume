use glam::Vec2;

/// A rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    position: Vec2,
    size: Vec2,
}

impl Rectangle {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn translated(&self, translation: Vec2) -> Self {
        Self {
            position: self.position + translation,
            ..*self
        }
    }

    pub fn scaled(&self, scale: Vec2) -> Self {
        Self {
            size: self.size * scale,
            ..*self
        }
    }
}
