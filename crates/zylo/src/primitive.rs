use glam::Vec2;

mod rectangle;

pub use rectangle::Rectangle;

/// A primitive shape.
///
/// Rendering backends may use SDFs or other specialized
/// techniques to render these shapes more precisely and efficiently
/// than they would had the shapes been approximated using Bezier paths.
///
/// # Transformations
/// Some transformations can be directly applied to `Primitive`s while maintaining their
/// primitive nature. These include:
/// * translation
/// * scaling, with a few notes:
///     * If the scale factor along the axes differ, then rounded rectangle radii are
///       scaled by the x-axis scale factor. The y-axis factor is ignored.
///     * If the scale factor along the axes differ, then a circle becomes an ellipse.
///
/// Rotation can be supported for 90-degree increments, but this is unimplemented.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Primitive {
    Rectangle(Rectangle),
    RoundedRectangle(RoundedRectangle),
    Circle(Circle),
    Ellipse(Ellipse),
}

impl Primitive {
    pub fn translated(&self, translation: Vec2) -> Self {
        match self {
            Primitive::Rectangle(rect) => Primitive::Rectangle(rect.translated(translation)),
            Primitive::RoundedRectangle(rounded_rect) => {
                Primitive::RoundedRectangle(rounded_rect.translated(translation))
            }
            Primitive::Circle(circle) => Primitive::Circle(circle.translated(translation)),
            Primitive::Ellipse(ellipse) => Primitive::Ellipse(ellipse.translated(translation)),
        }
    }

    pub fn scaled(&self, scale: Vec2) -> Self {
        match self {
            Primitive::Rectangle(rect) => Primitive::Rectangle(rect.scaled(scale)),
            Primitive::RoundedRectangle(rounded_rect) => {
                Primitive::RoundedRectangle(rounded_rect.scaled(scale))
            }
            Primitive::Circle(circle) => circle.scaled(scale),
            Primitive::Ellipse(ellipse) => Primitive::Ellipse(ellipse.scaled(scale)),
        }
    }
}

/// A rectangle with rounded corners.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RoundedRectangle {
    rectangle: Rectangle,
    border_radii: BorderRadii,
}

impl RoundedRectangle {
    pub fn new(rectangle: Rectangle, border_radii: BorderRadii) -> Self {
        Self {
            rectangle,
            border_radii,
        }
    }

    pub fn rectangle(&self) -> Rectangle {
        self.rectangle
    }

    pub fn border_radii(&self) -> BorderRadii {
        self.border_radii
    }

    pub fn translated(&self, translation: Vec2) -> Self {
        Self {
            rectangle: self.rectangle.translated(translation),
            ..*self
        }
    }

    pub fn scaled(&self, scale: Vec2) -> Self {
        Self {
            rectangle: self.rectangle.scaled(scale),
            border_radii: self.border_radii.scaled(scale),
        }
    }
}

/// The rounding radius applied to each corner of a rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BorderRadii {
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
}

impl BorderRadii {
    /// All corners have the same border radius.
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Explicitly set each corner's radius.
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn top_left(&self) -> f32 {
        self.top_left
    }

    pub fn top_right(&self) -> f32 {
        self.top_right
    }

    pub fn bottom_left(&self) -> f32 {
        self.bottom_left
    }

    pub fn bottom_right(&self) -> f32 {
        self.bottom_right
    }

    pub fn scaled(&self, scale: Vec2) -> Self {
        // If the two scales differ, we just pick the x-axis for now.
        let scale = scale.x;
        Self {
            top_left: self.top_left * scale,
            top_right: self.top_right * scale,
            bottom_right: self.bottom_right * scale,
            bottom_left: self.bottom_left * scale,
        }
    }
}

/// A circle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Circle {
    center: Vec2,
    radius: f32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn center(&self) -> Vec2 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn translated(&self, translation: Vec2) -> Self {
        Self {
            center: self.center + translation,
            ..*self
        }
    }

    /// Scales the circle. If the scales along the axes
    /// differ, then the circle becomes an ellipse.
    pub fn scaled(&self, scale: Vec2) -> Primitive {
        if scale.x == scale.y {
            Primitive::Circle(Self {
                radius: self.radius * scale.x,
                ..*self
            })
        } else {
            Primitive::Ellipse(Ellipse::from_circle(*self).scaled(scale))
        }
    }
}

/// An ellipse.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ellipse(Rectangle);

impl Ellipse {
    /// Creates an ellipse from its bounding rectangle.
    pub fn from_rectangle(rect: Rectangle) -> Self {
        Self(rect)
    }

    /// Creates an ellipse from a circle.
    pub fn from_circle(circle: Circle) -> Self {
        Self::from_rectangle(Rectangle::new(
            circle.center() - circle.radius(),
            circle.center() + circle.radius(),
        ))
    }

    pub fn rectangle(&self) -> Rectangle {
        self.0
    }

    pub fn translated(self, translation: Vec2) -> Self {
        Self(self.0.translated(translation))
    }

    pub fn scaled(self, scale: Vec2) -> Self {
        Self(self.0.scaled(scale))
    }
}

impl From<Rectangle> for Primitive {
    fn from(r: Rectangle) -> Self {
        Primitive::Rectangle(r)
    }
}

impl From<RoundedRectangle> for Primitive {
    fn from(r: RoundedRectangle) -> Self {
        Primitive::RoundedRectangle(r)
    }
}

impl From<Circle> for Primitive {
    fn from(c: Circle) -> Self {
        Primitive::Circle(c)
    }
}

impl From<Ellipse> for Primitive {
    fn from(e: Ellipse) -> Self {
        Primitive::Ellipse(e)
    }
}
