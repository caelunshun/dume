use std::{f32::consts::TAU, time::Instant};

use noise::{Fbm, MultiFractal, NoiseFn};
use winit::window::Window;
use zylo::{glam::vec2, Canvas, Color, GradientStop, Path, Vec2};
use zylo_app::Render;

struct App {
    start_time: Instant,
}

impl Render for App {
    fn render(&mut self, canvas: &mut Canvas, target_size: Vec2) {
        canvas.translate(target_size / 2.);

        let time = self.start_time.elapsed().as_secs_f32();

        let noise = Fbm::new().set_frequency(0.1);
        let noise_offset = time * 2.;

        let mut path = Path::builder();
        let stops = 256;
        let stop_angle = TAU / stops as f32;

        for i in 0..stops {
            let angle = stop_angle * i as f32;
            let point = vec2(angle.cos(), angle.sin()) + Vec2::splat(noise_offset);
            let radius = (noise.get([point.x as f64, point.y as f64]) as f32).abs() * 1500.;

            let x = angle.cos() * radius;
            let y = angle.sin() * radius;

            if i == 0 {
                path = path.move_to(vec2(x, y));
            } else {
                path = path.line_to(vec2(x, y));
            }
        }
        let path = path.close();

        canvas
            .fill_path(&path)
            .linear_gradient(
                -target_size / 2.,
                target_size / 2.,
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
