use std::{env, fs::File, io::BufReader, process};

use dume::Context;
use dume_video::Video;
use dume_winit::{block_on, Application, DumeWinit};
use glam::Vec2;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

struct App {
    video: Video,
}

impl App {
    pub fn new(cx: &Context) -> Self {
        let path = env::args().nth(1).expect("usage: play_video <path>");
        let file =
            BufReader::new(File::open(&path).expect("failed to open video file for reading"));
        Self {
            video: Video::new(cx, file).expect("failed to read video file header - make sure it's in IVF format and uses a VP9 codec")
        }
    }
}

impl Application for App {
    fn draw(&mut self, canvas: &mut dume::Canvas) {
        let width = canvas.size().x;
        if self.video.draw(canvas, Vec2::ZERO, width, 1.).is_err() {
            process::exit(0);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Dume Video Player")
        .with_inner_size(LogicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    block_on(async move {
        let dume = DumeWinit::new(window).await;
        let app = App::new(dume.context());
        dume.run(event_loop, app);
    });
}
