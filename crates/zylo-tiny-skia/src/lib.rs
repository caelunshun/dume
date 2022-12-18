//! A software rendering backend for `zylo` that uses [`tiny-skia`](https://docs.rs/tiny-skia).

use std::mem;

use slotmap::SlotMap;
use tiny_skia::{
    ClipMask, LinearGradient, Paint, PathBuilder, Pixmap, Point, Rect, Shader, SpreadMode, Stroke,
    StrokeDash, Transform,
};
use zylo::{
    glam::Affine2, Backend, Color, Command, CommandStream, FillRule, GradientStop, LayerId,
    LayerInfo, LineCap, LineJoin, PathSegment, Primitive, RoundedRectangle, StrokeSettings, Vec2,
};

/// A `tiny-skia` rendering backend.
#[derive(Default)]
pub struct TinySkiaBackend {
    renderer: Renderer,
    layers: SlotMap<LayerId, Layer>,
}

impl TinySkiaBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Backend for TinySkiaBackend {
    fn create_layer(&mut self, info: LayerInfo) -> LayerId {
        self.layers.insert(Layer {
            pixmap: Pixmap::new(info.physical_width(), info.physical_height())
                .expect("invalid layer dimensions"),
            info,
        })
    }

    fn layer_info(&self, id: LayerId) -> Option<&LayerInfo> {
        self.layers.get(id).map(|layer| &layer.info)
    }

    fn render_to_layer(&mut self, layer: LayerId, commands: CommandStream) {
        let layer = &mut self.layers[layer];
        self.renderer.render_to_layer(layer, commands);
    }
}

struct Layer {
    pixmap: Pixmap,
    info: LayerInfo,
}

enum CurrentShader {
    Solid(tiny_skia::Color),
    LinearGradient {
        stops: Vec<tiny_skia::GradientStop>,
        start: Vec2,
        end: Vec2,
    },
}

struct Renderer {
    shader: CurrentShader,
    object_transform: Transform,
    paint_transform: Transform,
    path_builder: PathBuilder,
    dashes: Vec<f32>,
    clip_mask: ClipMask,
    clip_mask_enabled: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            shader: CurrentShader::Solid(tiny_skia::Color::WHITE),
            object_transform: Transform::identity(),
            paint_transform: Transform::identity(),
            path_builder: PathBuilder::new(),
            dashes: Vec::new(),
            clip_mask: ClipMask::new(),
            clip_mask_enabled: false,
        }
    }
}

impl Renderer {
    pub fn render_to_layer(&mut self, layer: &mut Layer, commands: CommandStream) {
        for command in commands {
            self.execute_command(layer, command);
        }
        self.reset();
    }

    fn reset(&mut self) {
        self.path_builder.clear();
        self.dashes.clear();
        self.object_transform = Transform::identity();
        self.paint_transform = Transform::identity();
        self.clip_mask_enabled = false;
        self.shader = CurrentShader::Solid(tiny_skia::Color::WHITE);
    }

    fn paint(&self) -> Paint {
        Paint {
            anti_alias: true,
            shader: match &self.shader {
                CurrentShader::Solid(color) => Shader::SolidColor(*color),
                CurrentShader::LinearGradient { stops, start, end } => LinearGradient::new(
                    convert_point(*start),
                    convert_point(*end),
                    stops.clone(),
                    SpreadMode::Pad,
                    Transform::identity(),
                )
                .expect("invalid linear gradient"),
            },
            ..Default::default()
        }
    }

    fn object_transform(&self, hidpi_factor: f32) -> Transform {
        self.object_transform.pre_scale(hidpi_factor, hidpi_factor)
    }

    fn push_primitive_to_path(&mut self, primitive: Primitive) {
        match primitive {
            Primitive::Rectangle(rect) => self.path_builder.push_rect(
                rect.position().x,
                rect.position().y,
                rect.size().x,
                rect.size().y,
            ),
            Primitive::RoundedRectangle(rounded_rect) => self.push_rounded_rect(rounded_rect),
            Primitive::Circle(circle) => {
                self.path_builder
                    .push_circle(circle.center().x, circle.center().y, circle.radius())
            }
            Primitive::Ellipse(ellipse) => self.path_builder.push_oval(
                Rect::from_xywh(
                    ellipse.rectangle().position().x,
                    ellipse.rectangle().position().y,
                    ellipse.rectangle().size().x,
                    ellipse.rectangle().size().y,
                )
                .expect("invalid ellipse bounds"),
            ),
        }
    }

    fn push_rounded_rect(&mut self, _rounded_rect: RoundedRectangle) {
        todo!()
    }

    fn execute_command(&mut self, layer: &mut Layer, command: Command) {
        match command {
            Command::UseSolidPaint(color) => {
                self.shader = CurrentShader::Solid(convert_color(color));
            }
            Command::UseLinearGradientPaint { start, end } => {
                self.shader = CurrentShader::LinearGradient {
                    stops: Vec::new(),
                    start,
                    end,
                };
            }
            Command::PushGradientStop(stop) => match &mut self.shader {
                CurrentShader::LinearGradient { stops, .. } => {
                    stops.push(convert_gradient_stop(stop))
                }
                _ => panic!("push gradient stop when paint is not a gradient"),
            },
            Command::SetObjectTransform(trans) => self.object_transform = convert_transform(trans),
            Command::SetPaintTransform(trans) => self.paint_transform = convert_transform(trans),
            Command::ClearPath => self.path_builder.clear(),
            Command::PushPathSegment(segment) => match segment {
                PathSegment::MoveTo(pos) => self.path_builder.move_to(pos.x, pos.y),
                PathSegment::LineTo(pos) => self.path_builder.line_to(pos.x, pos.y),
                PathSegment::QuadTo { control, end } => self
                    .path_builder
                    .quad_to(control.x, control.y, end.x, end.y),
                PathSegment::CubicTo {
                    control1,
                    control2,
                    end,
                } => self
                    .path_builder
                    .cubic_to(control1.x, control1.y, control2.x, control2.y, end.x, end.y),
                PathSegment::Close => self.path_builder.close(),
            },
            Command::SetClipToPath { fill_rule } => {
                self.set_clip_to_path(fill_rule, layer);
            }
            Command::SetClipToPrimitive { primitive } => {
                self.push_primitive_to_path(primitive);
                self.set_clip_to_path(FillRule::default(), layer);
            }
            Command::ClearClip => {
                self.clip_mask_enabled = false;
                // ClipMask clears itself automatically the next time set_path is called
            }
            Command::ClearDashPairs => self.dashes.clear(),
            Command::PushDashPair(pair) => {
                self.dashes.extend([pair.on(), pair.off()]);
            }
            Command::FillPath { fill_rule } => self.fill_path(layer, fill_rule),
            Command::FillPrimitive { primitive } => {
                self.push_primitive_to_path(primitive);
                self.fill_path(layer, FillRule::EvenOdd)
            }
            Command::StrokePath { stroke_settings } => self.stroke_path(&stroke_settings, layer),
            Command::StrokePrimitive {
                stroke_settings,
                primitive,
            } => {
                self.push_primitive_to_path(primitive);
                self.stroke_path(&stroke_settings, layer);
            }
        }
    }

    fn clip_mask(&self) -> Option<&ClipMask> {
        self.clip_mask_enabled.then_some(&self.clip_mask)
    }

    fn fill_path(&mut self, layer: &mut Layer, fill_rule: FillRule) {
        self.with_current_path(|this, path| {
            layer.pixmap.fill_path(
                &path,
                &this.paint(),
                convert_fill_rule(fill_rule),
                this.object_transform(layer.info.hidpi_factor()),
                this.clip_mask(),
            );
            path
        });
    }

    fn stroke_path(&mut self, settings: &StrokeSettings, layer: &mut Layer) {
        self.with_current_path(|this, path| {
            let dash = if this.dashes.is_empty() {
                None
            } else {
                Some(
                    StrokeDash::new(mem::take(&mut this.dashes), settings.dash_offset)
                        .expect("invalid dashes"),
                )
            };
            layer.pixmap.stroke_path(
                &path,
                &this.paint(),
                &Stroke {
                    width: settings.width,
                    line_cap: convert_line_cap(settings.line_cap),
                    line_join: convert_line_join(settings.line_join),
                    dash,
                    ..Default::default()
                },
                this.object_transform(layer.info.hidpi_factor()),
                this.clip_mask(),
            );
            path
        });
    }

    fn set_clip_to_path(&mut self, fill_rule: FillRule, layer: &Layer) {
        self.with_current_path(|this, mut path| {
            path = path
                .transform(this.object_transform)
                .expect("invalid transform");
            this.clip_mask.set_path(
                layer.pixmap.width(),
                layer.pixmap.height(),
                &path,
                convert_fill_rule(fill_rule),
                true,
            );
            this.clip_mask_enabled = true;
            path
        });
    }

    fn with_current_path(
        &mut self,
        callback: impl FnOnce(&mut Self, tiny_skia::Path) -> tiny_skia::Path,
    ) {
        let builder = mem::take(&mut self.path_builder);
        let mut path = builder.finish().expect("attempted to render invalid path");
        path = callback(self, path);

        // Reuse the path builder's allocated space.
        // Note that this clears the builder, meaning a subsequent
        // draw command will use an empty path. However, the Canvas
        // always builds a path before every draw command, so
        // we need not worry. (Though this is an internal `zylo` implementation
        // detail.)
        self.path_builder = path.clear();
    }
}

fn convert_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha())
}

fn convert_gradient_stop(stop: GradientStop) -> tiny_skia::GradientStop {
    tiny_skia::GradientStop::new(stop.position(), convert_color(stop.color()))
}

fn convert_transform(transform: Affine2) -> Transform {
    let cols = transform.to_cols_array();
    Transform::from_row(cols[0], cols[1], cols[2], cols[3], cols[4], cols[5])
}

fn convert_point(point: Vec2) -> Point {
    Point::from_xy(point.x, point.y)
}

fn convert_line_cap(cap: LineCap) -> tiny_skia::LineCap {
    match cap {
        LineCap::Butt => tiny_skia::LineCap::Butt,
        LineCap::Round => tiny_skia::LineCap::Round,
        LineCap::Square => tiny_skia::LineCap::Square,
    }
}

fn convert_line_join(join: LineJoin) -> tiny_skia::LineJoin {
    match join {
        LineJoin::Miter => tiny_skia::LineJoin::Miter,
        LineJoin::Round => tiny_skia::LineJoin::Round,
        LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
    }
}

fn convert_fill_rule(rule: FillRule) -> tiny_skia::FillRule {
    match rule {
        FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
        FillRule::NonZero => tiny_skia::FillRule::Winding,
    }
}
