use std::{
    io::Read,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use dume::{yuv::Size, Canvas, Context, YuvTexture};
use flume::{Receiver, Sender};
use glam::{uvec2, Vec2};
use vp9::{
    ivf::{IvfDemuxer, IvfError},
    Frame, Vp9Decoder,
};

/// A high-level utility to render a video onto a `Canvas`.
///
/// The video is decoded from a video file. Currently, this library
/// only supports IVF-format files encoded with VP9. `dume-video` uses
/// [`vp9-rs`](https://github.com/caelunshun/vp9-rs) for decoding and demuxing.
///
/// The video decoding runs on a separate thread.
pub struct Video {
    texture: Arc<YuvTexture>,
    events: Receiver<Event>,
    is_finished: bool,
}

impl Video {
    pub fn new<R: Read + Send + 'static>(cx: &Context, reader: R) -> Result<Self, IvfError> {
        let demuxer = IvfDemuxer::new(reader)?;

        let texture = Arc::new(cx.create_yuv_texture(
            uvec2(demuxer.header().width, demuxer.header().height),
            Size::Full,
            Size::Half,
            Size::Half,
        )); // assume YUV420p for now (vp9-rs does the same)

        let (events_tx, events) = flume::unbounded();

        let texture2 = Arc::clone(&texture);
        thread::Builder::new()
            .name("video-decoder".to_owned())
            .spawn(move || {
                run_decoder_thread(&texture2, demuxer, events_tx);
            })
            .expect("failed to spawn video decoding thread");

        Ok(Self {
            texture,
            events,
            is_finished: false,
        })
    }

    /// Renders the current frame of the video onto a canvas.
    ///
    /// `width` is the size of the video on the X axis. The height is
    /// computed from the aspect ratio.
    ///
    /// Does nothing if the video is finished or has triggered an error.
    pub fn draw(
        &mut self,
        canvas: &mut Canvas,
        pos: Vec2,
        width: f32,
        alpha: f32,
    ) -> Result<(), Error> {
        if self.is_finished {
            return Err(Error::VideoEnded);
        }

        for event in self.events.try_iter() {
            self.is_finished = true;
            match event {
                Event::DemuxError(e) => return Err(Error::DemuxError(e)),
                Event::CodecError(e) => return Err(Error::CodecError(e)),
                Event::VideoEnded => return Err(Error::VideoEnded),
            }
        }

        canvas.draw_yuv_texture(&self.texture, pos, width, alpha);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("video has ended")]
    VideoEnded,
    #[error("codec error: {0}")]
    CodecError(vp9::Error),
    #[error("demuxing error: {0}")]
    DemuxError(IvfError),
}

enum Event {
    DemuxError(IvfError),
    CodecError(vp9::Error),
    VideoEnded,
}

fn run_decoder_thread(
    texture: &YuvTexture,
    mut demuxer: IvfDemuxer<impl Read>,
    events: Sender<Event>,
) {
    let start_time = Instant::now();
    let time_base = demuxer.header().time_base_num as f64 / demuxer.header().time_base_denom as f64;

    let mut frame = Frame::new(demuxer.header().width, demuxer.header().height);

    let mut decoder = Vp9Decoder::new();

    loop {
        match demuxer.next_frame() {
            Ok(Some(f)) => {
                if let Err(e) = decoder.decode(f.data) {
                    events.send(Event::CodecError(e)).ok();
                    return;
                }

                while decoder.next_frame(&mut frame).unwrap() {}

                // Wait until we should submit the frame, then upload to the GPU texture.
                let current_time = start_time.elapsed().as_secs_f64();
                let target_time = f.timestamp as f64 * time_base;

                if target_time > current_time {
                    thread::sleep(Duration::from_secs_f64(target_time - current_time));
                }

                texture.update(frame.y_plane(), frame.u_plane(), frame.v_plane());
            }
            Ok(None) => break,
            Err(e) => {
                events.send(Event::DemuxError(e)).ok();
                return;
            }
        }

        if events.is_disconnected() {
            return;
        }
    }

    events.send(Event::VideoEnded).ok();
}
