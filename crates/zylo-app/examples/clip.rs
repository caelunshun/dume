use std::{f32::consts::TAU, time::Instant};

use winit::window::Window;
use zylo::{glam::vec2, Circle, Color, GradientStop, Rectangle, Vec2};
use zylo_app::Render;

struct App {
    start_time: Instant,
}

impl Render for App {
    fn render(&mut self, canvas: &mut zylo::Canvas, target_size: zylo::Vec2) {
        let time = self.start_time.elapsed().as_secs_f32();
        let rect = Rectangle::new(target_size / 2., vec2(200., target_size.y - 200.));
        canvas.clip_with_primitive(Circle::new(target_size / 2., 300.));
        canvas
            .translate(rect.position())
            .rotate(time * TAU / 8.)
            .fill_primitive(Rectangle::new(Vec2::ZERO, rect.size()))
            .linear_gradient(
                Vec2::ZERO,
                rect.size(),
                [
                    GradientStop::new(0., Color::rgb(200, 180, 30)),
                    GradientStop::new(0.5, Color::rgb(30, 180, 200)),
                    GradientStop::new(1., Color::rgb(180, 30, 200)),
                ],
            )
            .draw();
    }
}

fn main() {
    zylo_app::run(
        |e| Window::new(&e).unwrap(),
        App {
            start_time: Instant::now(),
        },
    );
}
