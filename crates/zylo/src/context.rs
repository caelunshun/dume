use crate::{backend::ErasedBackend, layer::Layer};

/// A `zylo` rendering context.
///
/// Wraps a backend implementation.

pub struct Context {
    backend: Box<dyn ErasedBackend>,
}

impl Context {
    pub fn new(backend: impl Into<Box<dyn ErasedBackend>>) -> Self {
        Self {
            backend: backend.into(),
        }
    }

    pub fn create_layer(
        &self,
        physical_width: u32,
        physical_height: u32,
        hidpi_factor: f32,
    ) -> Layer {
        Layer::new(self, physical_width, physical_height, hidpi_factor)
    }

    pub fn backend(&self) -> &dyn ErasedBackend {
        &*self.backend
    }

    pub fn backend_mut(&mut self) -> &mut dyn ErasedBackend {
        &mut *self.backend
    }
}
