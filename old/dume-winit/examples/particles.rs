use dume::{Canvas, Srgba};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, IVec2, Vec2};
use instant::Instant;
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::Rng;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Srgba<u8>,
    is_circle: bool,
}

struct App {
    last_time: Instant,
    particles: Vec<Particle>,
    velocity_field: Vec<Vec<Vec2>>,
}

impl App {
    fn update_particles(&mut self) {
        let dt = self.last_time.elapsed().as_secs_f32();
        self.last_time = Instant::now();

        for particle in &mut self.particles {
            particle.pos += particle.vel * dt;
            let pos = particle
                .pos
                .as_ivec2()
                .clamp(IVec2::splat(0), IVec2::new(1919, 1079));
            particle.vel += self.velocity_field[pos.x as usize][pos.y as usize] * dt;
        }
    }
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        self.update_particles();

        for particle in &self.particles {
            canvas.solid_color(particle.color);
            if particle.is_circle {
                canvas.circle(particle.pos, 10.);
            } else {
                canvas.rect(particle.pos, Vec2::splat(20.));
            }
            canvas.fill();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    block_on(async move {
        let size = window.inner_size().to_logical(window.scale_factor());
        let dume = DumeWinit::new(window).await;
        let app = App {
            last_time: Instant::now(),
            particles: init_particles(size),
            velocity_field: init_velocity_field(),
        };
        dume.run(event_loop, app);
    });
}

fn init_particles(size: LogicalSize<u32>) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    (0..10_000)
        .map(|_| Particle {
            pos: vec2(rng.gen(), rng.gen()) * vec2(size.width as f32, size.height as f32) / 2.
                + vec2(size.width as f32, size.height as f32) / 4.,
            vel: Vec2::ZERO,
            color: Srgba::new(rng.gen(), rng.gen(), rng.gen(), 180),
            is_circle: rng.gen_bool(0.3),
        })
        .collect()
}

fn init_velocity_field() -> Vec<Vec<Vec2>> {
    let noise_a = Fbm::new().set_frequency(0.1).set_seed(100);
    let noise_b = Fbm::new().set_frequency(0.1).set_seed(500);
    let mut grid = vec![vec![Vec2::ZERO; 1080]; 1920];

    for x in 0..1920 {
        for y in 0..1080 {
            let a = noise_a.get([x as f64, y as f64]);
            let b = noise_b.get([x as f64, y as f64]);
            let mut vel = vec2(a as f32 * 20., b as f32 * 20.);
            if vel.length() < 1. {
                vel /= vel.length();
            }
            grid[x][y] = vel;
        }
    }

    grid
}
