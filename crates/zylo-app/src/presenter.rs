use winit::window::Window;
use zylo::{Context, Layer};

#[cfg(feature = "backend-tiny-skia")]
use softbuffer::GraphicsContext;
#[cfg(feature = "backend-tiny-skia")]
use zylo_tiny_skia::TinySkiaBackend;

use crate::BackendType;

/// Wraps a `winit::Window`. Allows presenting a Layer
/// to the window surface.
///
/// Currently, the Presenter needs to own the window
/// to ensure resources are dropped in the right order.
pub struct Presenter {
    inner: Inner,
}

impl Presenter {
    /// Creates a `Presenter` from the given window and `zylo::Context`.
    pub fn new(window: Window, context: &Context) -> Self {
        let backend = determine_backend(context);
        let inner = match backend {
            #[cfg(feature = "backend-tiny-skia")]
            BackendType::TinySkia => Inner::Software(unsafe {
                GraphicsContext::new(window).expect("failed to create graphics context")
            }),
            BackendType::Other => panic!("unsupported backend"),
        };
        Self { inner }
    }

    /// Presents a layer to the window.
    pub fn present(&mut self, layer: &Layer) {
        match &mut self.inner {
            #[cfg(feature = "backend-tiny-skia")]
            Inner::Software(context) => {
                context.set_buffer(
                    &layer.to_argb(),
                    layer.physical_width().try_into().unwrap(),
                    layer.physical_height().try_into().unwrap(),
                );
            }
        }
    }

    /// Gets the inner window.
    pub fn window(&self) -> &Window {
        match &self.inner {
            #[cfg(feature = "backend-tiny-skia")]
            Inner::Software(software) => software.window(),
        }
    }

    /// Mutably gets the inner window.
    pub fn window_mut(&mut self) -> &mut Window {
        match &mut self.inner {
            #[cfg(feature = "backend-tiny-skia")]
            Inner::Software(software) => software.window_mut(),
        }
    }
}

enum Inner {
    #[cfg(feature = "backend-tiny-skia")]
    Software(softbuffer::GraphicsContext<Window>),
}

fn determine_backend(context: &Context) -> BackendType {
    let backend = context.backend().as_any();

    #[cfg(feature = "backend-tiny-skia")]
    if backend.is::<TinySkiaBackend>() {
        return BackendType::TinySkia;
    }

    BackendType::Other
}
