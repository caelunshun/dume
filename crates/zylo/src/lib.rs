//! 2D rendering.

mod backend;
mod canvas;
mod color;
mod context;
mod layer;
mod path;
mod primitive;
mod types;
#[cfg(feature = "text")]
mod text; 

pub use backend::{
    command::{Command, CommandStream},
    Backend, BackendLayer, ErasedBackend,
};
pub use canvas::{Canvas, Fill, Stroke};
pub use color::Color;
pub use context::Context;
pub use glam::Vec2;
pub use layer::Layer;
pub use path::{Path, PathBuilder, PathSegment};
pub use primitive::{BorderRadii, Circle, Ellipse, Primitive, Rectangle, RoundedRectangle};
pub use types::{DashPair, FillRule, GradientStop, LineCap, LineJoin, StrokeSettings};

pub extern crate glam;
