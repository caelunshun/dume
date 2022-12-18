use crate::{LayerId, LayerInfo};

use self::command::CommandStream;

pub mod command;

/// A drawing backend that implements the `zylo` rendering model.
pub trait Backend: 'static {
    fn create_layer(&mut self, info: LayerInfo) -> LayerId;

    fn layer_info(&self, id: LayerId) -> Option<&LayerInfo>;

    fn render_to_layer(&mut self, layer: LayerId, commands: CommandStream);
}
