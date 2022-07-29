use std::{f32::consts::TAU, time::Instant};

use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use winit::window::Window;
use zylo::{
    glam::{vec2, vec4},
    Circle, Color, Ellipse, Primitive, Rectangle, Vec2,
};
use zylo_app::Render;

struct App {
    start_time: Instant,
}

impl Render for App {
    fn render(&mut self, canvas: &mut zylo::Canvas, target_size: zylo::Vec2) {
        let time = self.start_time.elapsed().as_secs_f32();
        canvas.translate(target_size / 2.);

        let radius = target_size.min_element() * 0.3;

        let size = 200.;
        let rect = Rectangle::new(Vec2::splat(-size / 2.), Vec2::splat(size));
        let circle = Circle::new(Vec2::ZERO, size);
        let ellipse = Ellipse::from_rectangle(Rectangle::new(
            Vec2::splat(-size / 2.),
            vec2(size * 2., size),
        ));

        let primitives: [Primitive; 3] = [rect.into(), circle.into(), ellipse.into()];
        let angle_delta = TAU / primitives.len() as f32;

        for (i, prim) in primitives.into_iter().enumerate() {
            let angle = angle_delta * i as f32 + (time * TAU * 0.3);
            let pos = vec2(angle.cos(), angle.sin()) * radius;

            let color_noise = Fbm::new().set_seed(i as u32).set_frequency(0.1);
            let color_noise_offset = (time + i as f32 * 4.) as f64;
            let color = Color::from_linear(vec4(
                color_noise
                    .get([color_noise_offset, color_noise_offset])
                    .abs() as f32,
                color_noise
                    .get([color_noise_offset + 1., color_noise_offset])
                    .abs() as f32,
                color_noise
                    .get([color_noise_offset + 2., color_noise_offset])
                    .abs() as f32,
                1.,
            ));

            let rot = time * TAU / 2.;

            canvas.with_save(|canvas| {
                canvas.translate(pos);
                canvas.rotate(rot);

                canvas.fill_primitive(prim).solid_color(color).draw();
            });
        }
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
