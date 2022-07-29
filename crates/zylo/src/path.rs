use std::fmt::Debug;

use glam::Vec2;

/// A vector path composed of line segments and Bezier curves.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    segments: Vec<PathSegment>,
}

impl Path {
    pub fn builder() -> PathBuilder {
        PathBuilder::new()
    }

    pub fn segments(&self) -> impl Iterator<Item = PathSegment> + '_ {
        self.segments.iter().copied()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathSegment {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo {
        control: Vec2,
        end: Vec2,
    },
    CubicTo {
        control1: Vec2,
        control2: Vec2,
        end: Vec2,
    },
    Close,
}

/// A builder for a [`PathHandle`].
///
/// Maintains a current "pen position," which is initially
/// set to the origin.
pub struct PathBuilder {
    path: Path,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            path: Path {
                segments: Vec::new(),
            },
        }
    }

    /// Moves the pen position to the given point without
    /// drawing a segment to it.
    pub fn move_to(mut self, point: Vec2) -> Self {
        self.push_segment(PathSegment::MoveTo(point));
        self
    }

    /// Adds a line segment from the pen position to the given
    /// point, then sets the pen position to `point`.
    pub fn line_to(mut self, point: Vec2) -> Self {
        self.push_segment(PathSegment::LineTo(point));
        self
    }

    /// Adds a quadratic Bezier curve from the pen position
    /// to `end` using the given control point. The pen
    /// position is set to `end`.
    pub fn quad_to(mut self, control: Vec2, end: Vec2) -> Self {
        self.push_segment(PathSegment::QuadTo { control, end });
        self
    }

    /// Adds a cubic Bezier curve from the pen position
    /// to `end` using the given control points. The pen position
    /// is set to `end`.
    pub fn cubic_to(mut self, control1: Vec2, control2: Vec2, end: Vec2) -> Self {
        self.push_segment(PathSegment::CubicTo {
            control1,
            control2,
            end,
        });
        self
    }

    /// Closes the path, then builds it.
    ///
    /// Closing a path adds a line segment to the initial point in the path.
    pub fn close(mut self) -> Path {
        self.push_segment(PathSegment::Close);
        self.build()
    }

    /// Builds the path and registers it with the context,
    /// returning a handle to it.
    pub fn build(self) -> Path {
        self.path
    }

    fn push_segment(&mut self, segment: PathSegment) {
        self.path.segments.push(segment);
    }
}
