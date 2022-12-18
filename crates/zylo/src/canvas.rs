use glam::{Affine2, Vec2};

use crate::{
    backend::command::{Command, CommandBuffer},
    primitive::Primitive,
    types::{DashPair, GradientStop, LineCap, LineJoin, StrokeSettings},
    Backend, Color, Context, FillRule, LayerId, Path,
};

/// A canvas to draw to.
///
/// Records a sequence of draw commands, then passes those
/// commands to the renderer.
///
/// The canvas maintains a _current transform_, which makes it stateful.
/// As a result, you want to ensure drawing operations happen in isolation.
/// A function that draws to a canvas, thus updating the canvas state, should
/// not affect any functions that come after it. To solve this problem, `zylo`
/// offers a save/restore API to create a stack of canvas states. See the methods
/// [`save`] and [`restore`].
///
/// Many methods return `self` to enable method chaining.
pub struct Canvas {
    commands: CommandBuffer,

    state_stack: Vec<State>,
    state: State,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new() -> Self {
        Self {
            commands: CommandBuffer::new(),
            state_stack: Vec::new(),
            state: State::default(),
        }
    }

    /// Translates the canvas.
    pub fn translate(&mut self, translation: Vec2) -> &mut Self {
        self.state.transform.translation += translation;
        self.emit_transform();
        self
    }

    /// Scales the canvas.
    pub fn scale(&mut self, scale: Vec2) -> &mut Self {
        self.state.transform = self.state.transform * Affine2::from_scale(scale);
        self.emit_transform();
        self
    }

    /// Rotates the canvas by the given angle in radians.
    pub fn rotate(&mut self, angle_in_radians: f32) -> &mut Self {
        self.state.transform = self.state.transform * Affine2::from_angle(angle_in_radians);
        self.emit_transform();
        self
    }

    /// Pushes the current transform onto the transform stack,
    /// allowing it to be restored later.
    pub fn save(&mut self) -> &mut Self {
        self.state_stack.push(self.state);
        self
    }

    /// Restores the next saved transform in the canvas's state stack.
    ///
    /// # Panics
    /// Panics if there is no state to pop. This happens only when
    /// `restore()` is called more times than `save()`.
    pub fn restore(&mut self) -> &mut Self {
        self.state = self
            .state_stack
            .pop()
            .expect("called Canvas::restore() at the top of the state stack");
        if !self.state.has_clip {
            self.clear_clip();
        }
        self.emit_transform();
        self
    }

    /// Calls `save()`, executes the closure, and then calls `restore()`.
    pub fn with_save(&mut self, f: impl FnOnce(&mut Self)) {
        self.save();
        f(self);
        self.restore();
    }

    /// Clips the rendered content to a path's bounds.
    ///
    /// Overrides any previous clip.
    pub fn clip_with_path(&mut self, path: &Path) -> &mut Self {
        self.set_path(path);
        self.cmd(Command::SetClipToPath {
            fill_rule: FillRule::EvenOdd,
        });
        self.state.has_clip = true;
        self
    }

    /// Clips the rendered content to a primitive.
    ///
    /// Overrides any previous clip.
    pub fn clip_with_primitive(&mut self, primitive: impl Into<Primitive>) -> &mut Self {
        self.cmd(Command::SetClipToPrimitive {
            primitive: primitive.into(),
        });
        self.state.has_clip = true;
        self
    }

    /// Clears the current clip.
    pub fn clear_clip(&mut self) -> &mut Self {
        self.cmd(Command::ClearClip);
        self.state.has_clip = false;
        self
    }

    /// Creates a builder to fill the given path.
    pub fn fill_path(&mut self, path: &Path) -> Fill {
        self.set_path(path);
        Fill::new(self, None)
    }

    /// Creates a builder to fill the given primitive.
    pub fn fill_primitive(&mut self, primitive: impl Into<Primitive>) -> Fill {
        Fill::new(self, Some(primitive.into()))
    }

    /// Creates a builder to stroke the given path.
    pub fn stroke_path(&mut self, path: &Path) -> Stroke {
        self.set_path(path);
        Stroke::new(self, None)
    }

    /// Creates a builder to stroke the given primitive.
    pub fn stroke_primitive(&mut self, primitive: impl Into<Primitive>) -> Stroke {
        Stroke::new(self, Some(primitive.into()))
    }

    /// Renders the canvas to the given `Layer`, flushing
    /// the draw command buffer.
    ///
    /// The canvas can be reused after this call.
    pub fn render_to_layer<B: Backend>(&mut self, context: &mut Context<B>, layer: LayerId) {
        context
            .backend_mut()
            .render_to_layer(layer, self.commands.to_stream());
        self.commands.clear();
        self.reset();
    }

    fn reset(&mut self) {
        self.state_stack.clear();
        self.state = State::default();
    }

    fn set_path(&mut self, path: &Path) {
        self.cmd(Command::ClearPath);
        for segment in path.segments() {
            self.cmd(Command::PushPathSegment(segment));
        }
    }

    fn set_solid_paint(&mut self, color: Color) {
        self.cmd(Command::UseSolidPaint(color));
    }

    fn set_linear_gradient_paint(
        &mut self,
        start: Vec2,
        end: Vec2,
        stops: impl Iterator<Item = GradientStop>,
    ) {
        self.cmd(Command::UseLinearGradientPaint { start, end });
        self.set_gradient_stops(stops);
    }

    fn set_gradient_stops(&mut self, stops: impl Iterator<Item = GradientStop>) {
        for stop in stops {
            self.cmd(Command::PushGradientStop(stop));
        }
    }

    fn emit_transform(&mut self) {
        let transform = self.state.transform;
        self.cmd(Command::SetObjectTransform(transform))
            .cmd(Command::SetPaintTransform(transform));
    }

    fn cmd(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }

    #[cfg(test)]
    fn take_commands(&mut self) -> Vec<Command> {
        let commands = self.commands.to_stream().collect();
        self.commands.clear();
        commands
    }
}

macro_rules! set_paint_fns {
    () => {
        /// Uses a solid color for the fill.
        pub fn solid_color(mut self, color: impl Into<Color>) -> Self {
            self.canvas.set_solid_paint(color.into());
            self.set_paint = true;
            self
        }

        /// Uses a linear gradient for the fill.
        pub fn linear_gradient(
            mut self,
            start: Vec2,
            end: Vec2,
            stops: impl IntoIterator<Item = GradientStop>,
        ) -> Self {
            self.canvas
                .set_linear_gradient_paint(start, end, stops.into_iter());
            self.set_paint = true;
            self
        }
    };
}

/// Builder-like API to fill a shape.
///
/// Allows configuring the following:
/// * the paint / shader to use - defaults to a
/// solid black paint
/// * the fill rule - defaults to EvenOdd
///
/// Call `draw()` to finish the draw operation.
#[must_use = "call Fill::draw() to finish the builder"]
pub struct Fill<'cv> {
    canvas: &'cv mut Canvas,
    primitive: Option<Primitive>,
    set_paint: bool,
    fill_rule: FillRule,
}

impl<'cv> Fill<'cv> {
    fn new(canvas: &'cv mut Canvas, primitive: Option<Primitive>) -> Self {
        Self {
            canvas,
            primitive,
            set_paint: false,
            fill_rule: FillRule::default(),
        }
    }

    set_paint_fns!();

    /// Sets the fill rule.
    ///
    /// Only matters for path filling. Primitives
    /// are filled the same regardless of fill rule.
    pub fn fill_rule(mut self, fill_rule: FillRule) -> Self {
        self.fill_rule = fill_rule;
        self
    }

    /// Draws the path.
    ///
    /// (Or rather, emits the command that causes the path to be drawn
    /// when `Canvas::render()` is called.)
    pub fn draw(mut self) {
        if !self.set_paint {
            self = self.solid_color(Color::WHITE);
        }

        let cmd = match self.primitive {
            Some(primitive) => Command::FillPrimitive { primitive },
            None => Command::FillPath {
                fill_rule: self.fill_rule,
            },
        };
        self.canvas.cmd(cmd);
    }
}

/// Builder-like API to stroke a shape.
///
/// Allows configuring the following:
/// * the paint - defaults to a solid white color
/// * the stroke width - defaults to 1.0
/// * the line cap - defaults to Butt
/// * the line join - defaults to Miter
/// * stroke dashes - default to none (indicating a full solid stroke)
///
/// Call `draw()` to finish the draw operation.
#[must_use = "call Stroke::draw() to finish the builder"]
pub struct Stroke<'cv> {
    canvas: &'cv mut Canvas,
    primitive: Option<Primitive>,
    settings: StrokeSettings,
    set_paint: bool,
    set_dashes: bool,
}

impl<'cv> Stroke<'cv> {
    fn new(canvas: &'cv mut Canvas, primitive: Option<Primitive>) -> Self {
        Self {
            canvas,
            primitive,
            settings: StrokeSettings::default(),
            set_paint: false,
            set_dashes: false,
        }
    }

    set_paint_fns!();

    /// Sets the stroke width.
    pub fn width(mut self, stroke_width: f32) -> Self {
        self.settings.width = stroke_width;
        self
    }

    /// Sets the line cap.
    pub fn line_cap(mut self, line_cap: LineCap) -> Self {
        self.settings.line_cap = line_cap;
        self
    }

    /// Sets the line join.
    pub fn line_join(mut self, line_join: LineJoin) -> Self {
        self.settings.line_join = line_join;
        self
    }

    /// Dashes the stroke, alternating over
    /// the given list of dash pairs.
    pub fn dash(mut self, offset: f32, dashes: impl IntoIterator<Item = DashPair>) -> Self {
        self.canvas.cmd(Command::ClearDashPairs);
        for dash in dashes {
            self.canvas.cmd(Command::PushDashPair(dash));
        }
        self.set_dashes = true;
        self.settings.dash_offset = offset;
        self
    }

    /// Draws the stroke.
    ///
    /// (Or rather, emits the command that causes the path to be drawn
    /// when `Canvas::render()` is called.)
    pub fn draw(mut self) {
        if !self.set_paint {
            self = self.solid_color(Color::WHITE);
        }

        let cmd = match self.primitive {
            Some(primitive) => Command::StrokePrimitive {
                stroke_settings: self.settings,
                primitive,
            },
            None => Command::StrokePath {
                stroke_settings: self.settings,
            },
        };
        self.canvas.cmd(cmd);

        // Clean up modified renderer state
        if self.set_dashes {
            self.canvas.cmd(Command::ClearDashPairs);
        }
    }
}

/// The state of the canvas.
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct State {
    transform: Affine2,
    // TODO allow saving/restoring clips without overhead.
    // For now, save/restore doesn't work with nested clips.
    has_clip: bool,
}

#[cfg(test)]
mod tests {
    use glam::vec2;

    use crate::path::PathSegment;

    use super::*;

    #[test]
    fn fill_path() {
        let mut canvas = Canvas::new();

        let path = Path::builder()
            .move_to(vec2(500., 500.))
            .line_to(vec2(1000., 1000.))
            .build();

        canvas.fill_path(&path).draw();

        assert_eq!(
            canvas.take_commands(),
            vec![
                Command::ClearPath,
                Command::PushPathSegment(PathSegment::MoveTo(vec2(500., 500.))),
                Command::PushPathSegment(PathSegment::LineTo(vec2(1000., 1000.))),
                Command::UseSolidPaint(Color::WHITE),
                Command::FillPath {
                    fill_rule: FillRule::default()
                }
            ]
        );
    }
}
