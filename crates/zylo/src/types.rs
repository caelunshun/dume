use crate::Color;

/// Determines how to fill paths will self-intersections.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}

impl Default for FillRule {
    fn default() -> Self {
        FillRule::EvenOdd
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl Default for LineCap {
    fn default() -> Self {
        LineCap::Butt
    }
}

impl Default for LineJoin {
    fn default() -> Self {
        LineJoin::Miter
    }
}

/// How to stroke a path.
///
/// Note that dashing parameters are stored separately.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StrokeSettings {
    /// Width of the path to stroke
    pub width: f32,
    /// How to cap the ends of open segments
    pub line_cap: LineCap,
    /// How to join segments together
    pub line_join: LineJoin,
    /// The offset of the first dash
    pub dash_offset: f32,
}

impl Default for StrokeSettings {
    fn default() -> Self {
        Self {
            width: 1.,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            dash_offset: 0.,
        }
    }
}

/// A dash pair applied to a stroke.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DashPair {
    on: f32,
    off: f32,
}

impl DashPair {
    /// Creates a dash pair indicating a drawn
    /// segment of length `on` followed by a hidden segment of
    /// length `off`.
    pub fn new(on: f32, off: f32) -> Self {
        Self { on, off }
    }

    /// Creates a dash pair with `on` and `off` equal to the same value,
    /// leading to evenly spaced dashes of the same length.
    pub fn splat(length: f32) -> Self {
        Self::new(length, length)
    }

    pub fn on(&self) -> f32 {
        self.on
    }

    pub fn off(&self) -> f32 {
        self.off
    }
}

/// A "stop" in a gradient, consisting
/// of a position (0.0..=1.0) along the gradient
/// and the color value at that position.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GradientStop {
    position: f32,
    color: Color,
}

impl GradientStop {
    pub fn new(position: f32, color: impl Into<Color>) -> Self {
        Self {
            position,
            color: color.into(),
        }
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
