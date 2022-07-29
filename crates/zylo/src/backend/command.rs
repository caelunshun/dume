use std::slice;

use glam::{Affine2, Vec2};

use crate::{
    path::PathSegment,
    primitive::Primitive,
    types::{DashPair, GradientStop, StrokeSettings},
    Color, FillRule,
};

/// A low-level command given to the backend renderer.
///
/// A draw operation involves a stream of `Command`s.
///
/// A command stream is designed to be _flattened_. Each `Command`
/// should represent one atomic unit, and it should not contain
/// heap-allocated vectors or other dynamically-sized data.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command {
    /// Sets the current paint to a solid fill.
    UseSolidPaint(Color),
    /// Sets the current paint to a linear gradient.
    ///
    /// Always followed in the stream by two or more `AddGradientStop` commands.
    UseLinearGradientPaint {
        start: Vec2,
        end: Vec2,
    },
    //// Adds a stop to the current gradient.
    PushGradientStop(GradientStop),

    /// Sets the transform applied to rendered
    /// paths and primitives.
    SetObjectTransform(Affine2),
    /// Sets the transform applied to paints (gradients,
    /// images, etc.)
    SetPaintTransform(Affine2),

    /// Clears the currently staged path
    ClearPath,
    /// Pushes a segment onto the current path
    PushPathSegment(PathSegment),

    /// Sets the clip to the currently staged path, then
    /// clears the staged path.
    SetClipToPath {
        fill_rule: FillRule,
    },
    /// Sets the clip to a primitive.
    SetClipToPrimitive {
        primitive: Primitive,
    },
    /// Clears the clip,
    ClearClip,

    /// Clears the current list of dash pairs
    ClearDashPairs,
    /// Appends a dash pair to the current list
    PushDashPair(DashPair),

    // Draw operations
    /// Fills the current path using the current configuration.
    FillPath {
        fill_rule: FillRule,
    },
    /// Fills a primitive using the current configuration.
    FillPrimitive {
        primitive: Primitive,
    },
    /// Strokes the current path using the current configuration.
    StrokePath {
        stroke_settings: StrokeSettings,
    },
    /// Strokes a primitive using the current configuration.
    StrokePrimitive {
        stroke_settings: StrokeSettings,
        primitive: Primitive,
    },
}

/// An immutable stream of `Command`s.
#[derive(Debug, Clone)]
pub struct CommandStream<'a> {
    commands: slice::Iter<'a, Command>,
}

impl<'a> Iterator for CommandStream<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.commands.next().copied()
    }
}

/// A buffer of `Command`s.
#[derive(Debug)]
pub(crate) struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }

    pub fn to_stream(&self) -> CommandStream {
        CommandStream {
            commands: self.commands.iter(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear()
    }
}
