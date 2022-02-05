use std::{
    f32::consts::{PI, TAU},
    time::Instant,
};

use dume::Canvas;
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::Window};

fn five_point_star(canvas: &mut Canvas, center: Vec2, outer_radius: f32, inner_radius: f32) {
    let angle_step = PI * 2. / 5.;

    for i in 0..5 {
        let outer_theta = angle_step * i as f32 - PI / 2.;
        let inner_theta = angle_step * (i as f32 + 0.5) - PI / 2.;

        let outer_pos = vec2(outer_theta.cos(), outer_theta.sin()) * outer_radius + center;
        let inner_pos = vec2(inner_theta.cos(), inner_theta.sin()) * inner_radius + center;

        if i == 0 {
            canvas.move_to(outer_pos);
        } else {
            canvas.line_to(outer_pos);
        }
        canvas.line_to(inner_pos);
    }

    // Close the path
    canvas.line_to(center - vec2(0., outer_radius));
}

struct App {
    start: Instant,
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let time = self.start.elapsed().as_secs_f32();
        let num_stars = 16;
        let center = canvas.size() / 2.;
        let radius = 200.;
        for i in 0..num_stars {
            let theta = (i as f32 / num_stars as f32) * TAU + time;
            let pos = vec2(theta.sin(), theta.cos()) * radius + center;
            canvas.begin_path();
            five_point_star(canvas, pos, 30., 15.);
            canvas
                .solid_color((190, 60, 210, u8::MAX))
                .fill()
                .solid_color((u8::MAX, u8::MAX, u8::MAX, u8::MAX))
                .stroke_width(1.)
                .stroke();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App {
            start: Instant::now(),
        };
        dume.run(event_loop, app);
    });
}
