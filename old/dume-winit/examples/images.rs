use std::fs;

use dume::{Canvas, SpriteRotate, TextureId};
use dume_winit::{block_on, Application, DumeWinit};
use glam::{vec2, Vec2};
use winit::{event_loop::EventLoop, window::Window};

struct App {
    image1: TextureId,
    image2: TextureId,
}

impl Application for App {
    fn draw(&mut self, canvas: &mut Canvas) {
        let size = canvas.size();
        canvas
            .draw_sprite(self.image1, Vec2::ZERO, size.x / 2.)
            .draw_sprite_with_rotation(
                self.image2,
                vec2(size.x / 2., 0.),
                size.x / 2.,
                SpriteRotate::Three,
            );
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let mut builder = dume.context().create_texture_set_builder();
        builder
            .add_texture(&fs::read("assets/image1.jpeg").unwrap(), "image1")
            .unwrap();
        builder
            .add_texture(&fs::read("assets/image2.jpeg").unwrap(), "image2")
            .unwrap();
        dume.context()
            .add_texture_set(builder.build(128, 8192).unwrap());

        let app = App {
            image1: dume.context().texture_for_name("image1").unwrap(),
            image2: dume.context().texture_for_name("image2").unwrap(),
        };
        dume.run(event_loop, app);
    });
}
