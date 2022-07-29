use std::time::Instant;

use winit::window::Window;
use zylo::{Canvas, Color, DashPair, GradientStop, LineCap, Rectangle, Vec2};
use zylo_app::Render;

struct App {
    start_time: Instant,
}

impl Render for App {
    fn render(&mut self, canvas: &mut Canvas, target_size: Vec2) {
        let time = self.start_time.elapsed().as_secs_f32();

        canvas.translate(target_size / 2.0);

        let size = Vec2::splat(500.);

        canvas
            .stroke_primitive(Rectangle::new(-size / 2., size))
            .linear_gradient(
                -size / 2.,
                size / 2.,
                [
                    GradientStop::new(0., Color::rgb(200, 180, 30)),
                    GradientStop::new(0.5, Color::rgb(30, 180, 200)),
                    GradientStop::new(1., Color::rgb(180, 30, 200)),
                ],
            )
            .dash(time * 100., [DashPair::splat(50.)])
            .width(10.)
            .line_cap(LineCap::Round)
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
