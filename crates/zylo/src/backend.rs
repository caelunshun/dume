use std::any::Any;

use crate::Color;

use self::command::CommandStream;

pub mod command;

/// A drawing backend that implements the `zylo` rendering model.
pub trait Backend: 'static {
    type Layer: BackendLayer;

    fn create_layer(
        &self,
        physical_width: u32,
        physical_height: u32,
        hidpi_factor: f32,
    ) -> Self::Layer;

    fn render_to_layer(&mut self, layer: &mut Self::Layer, commands: CommandStream);
}

/// A target surface for a rendering backend.
///
/// A type implementing `Layer` contains a 2D image
/// representing the rendered pixels.
pub trait BackendLayer: 'static {
    fn fill(&mut self, color: Color);

    fn to_argb(&self) -> Vec<u32>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Type-erased version of `Backend`.
pub trait ErasedBackend: 'static {
    fn create_layer(
        &self,
        physical_width: u32,
        physical_height: u32,
        hidpi_factor: f32,
    ) -> Box<dyn BackendLayer>;

    fn render_to_layer(&mut self, layer: &mut dyn BackendLayer, commands: CommandStream);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ErasedBackend for T
where
    T: Backend,
{
    fn create_layer(
        &self,
        physical_width: u32,
        physical_height: u32,
        hidpi_factor: f32,
    ) -> Box<dyn BackendLayer> {
        let layer =
            <T as Backend>::create_layer(self, physical_width, physical_height, hidpi_factor);
        Box::new(layer)
    }

    fn render_to_layer(&mut self, layer: &mut dyn BackendLayer, commands: CommandStream) {
        <T as Backend>::render_to_layer(
            self,
            layer
                .as_any_mut()
                .downcast_mut()
                .expect("layer type does not correspond to this backend"),
            commands,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
