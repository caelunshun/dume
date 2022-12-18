use crate::{Backend, LayerId, LayerInfo};

/// A `zylo` rendering context.
///
/// Wraps a backend implementation.
pub struct Context<B> {
    backend: B,
}

impl<B> Context<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn create_layer(&mut self, info: LayerInfo) -> LayerId {
        self.backend.create_layer(info)
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}
