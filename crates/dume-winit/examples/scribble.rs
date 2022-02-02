use std::f32::consts::{PI, TAU};

use dume::{Canvas, StrokeCap};
use dume_winit::{block_on, Application, DumeWinit};
use glam::vec2;
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::{prelude::StdRng, Rng, SeedableRng};
use std::time::Instant;
use winit::{event_loop::EventLoop, window::Window};

struct App {
    seed: u64,
    start: Instant,
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let mut rng = StdRng::seed_from_u64(self.seed);

        let size = canvas.size();
        let center = size / 2.;
        let base_radius = size / 2.5;
        let radius_variance = base_radius * 0.3;

        let num_stops = 1024;

        let noise = Fbm::new()
            .set_seed(rng.gen())
            .set_frequency((PI / 2.) as f64);

        let time = self.start.elapsed().as_secs_f32() * 0.2;

        canvas.begin_path();
        for stop in 0..=num_stops {
            let mut theta = (stop as f32 / num_stops as f32) * TAU;
            if stop == num_stops {
                theta = 0.;
            }
            let r = base_radius + noise.get([theta as f64, time as f64]) as f32 * radius_variance;
            let pos = center + vec2(theta.cos(), theta.sin()) * r;
            if stop == 0 {
                canvas.move_to(pos);
            } else {
                canvas.line_to(pos);
            }
        }

        canvas
            .stroke_cap(StrokeCap::Round)
            .stroke_width(3.)
            .solid_color((60, 190, 150, u8::MAX))
            .stroke();
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App {
            seed: rand::random(),
            start: Instant::now(),
        };
        dume.run(event_loop, app);
    });
}
