//! 2D rendering.

mod backend;
mod canvas;
mod color;
mod context;
mod layer;
mod path;
mod primitive;
#[cfg(feature = "text")]
pub mod text;
mod types;

pub use backend::{
    command::{Command, CommandStream},
    Backend,
};
pub use canvas::{Canvas, Fill, Stroke};
pub use color::Color;
pub use context::Context;
pub use glam::Vec2;
pub use layer::{LayerId, LayerInfo};
pub use path::{Path, PathBuilder, PathSegment};
pub use primitive::{BorderRadii, Circle, Ellipse, Primitive, Rectangle, RoundedRectangle};
#[cfg(feature = "text")]
#[doc(inline)]
pub use text::{
    builder::{BuildText, IntoSpan},
    layout::TextGalley,
    span::{Span, Text},
    style::{FontFamily, Style},
    Weight,
};
pub use types::{DashPair, FillRule, GradientStop, LineCap, LineJoin, StrokeSettings};

pub extern crate glam;
