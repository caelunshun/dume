use glam::{vec2, Affine2, Vec2};

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

    pub fn empty() -> Self {
        Self {
            pos: Vec2::ZERO,
            size: Vec2::ZERO,
        }
    }

    pub fn is_empty(self) -> bool {
        self.size == Vec2::ZERO
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

    pub fn bbox_transformed(self, transform: Affine2) -> Self {
        let points = [
            self.pos,
            self.pos + vec2(0., self.size.y),
            self.pos + vec2(self.size.x, 0.),
            self.pos + self.size,
        ]
        .map(|p| transform.transform_point2(p));

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(-f32::INFINITY);
        for point in points {
            min = min.min(point);
            max = max.max(point);
        }

        Self {
            pos: min,
            size: max - min,
        }
    }

    pub fn normalize_negative_size(&mut self) {
        if self.size.x < 0. {
            self.pos.x += self.size.x;
            self.size.x = self.size.x.abs();
        }

        if self.size.y < 0. {
            self.pos.y += self.size.y;
            self.size.y = self.size.y.abs();
        }
    }

    pub fn bottom_right(self) -> Vec2 {
        self.pos + self.size
    }

    pub fn overlaps(self, other: Rect) -> bool {
        !(self.pos.x > other.pos.x + other.size.x
            || other.pos.x > self.pos.x + self.size.x
            || self.pos.y > other.pos.y + other.size.y
            || other.pos.y > self.pos.y + self.size.y)
    }

    /// Computes the intersection of the regions bounded by `self` and `other`.
    ///
    /// If the two rectangles do not overlap, this function returns `None`.
    pub fn intersection(self, other: Rect) -> Option<Rect> {
        if !self.overlaps(other) {
            None
        } else {
            let pos = self.pos.max(other.pos);
            Some(Rect {
                pos,
                size: self.bottom_right().min(other.bottom_right()) - pos,
            })
        }
    }

    /// Computes the union of the regions bounded by `self` and `other`.
    ///
    /// This function returns a rectangle that contains both `self` and `other`.
    pub fn union(self, other: Rect) -> Rect {
        if self.is_empty() {
            other
        } else if other.is_empty() {
            self
        } else {
            let pos = self.pos.min(other.pos);
            Rect {
                pos,
                size: self.bottom_right().max(other.bottom_right()) - pos,
            }
        }
    }

    /// Returns a rectangle with borders expanded
    /// by the given (possibly negative, yielding a shrink)
    /// amount.
    pub fn expanded(&self, amount: f32) -> Self {
        Self {
            pos: self.pos - Vec2::splat(amount),
            size: self.size + 2. * Vec2::splat(amount),
        }
    }
}
