use crate::{Backend, LayerId, LayerInfo};

#[cfg(feature = "text")]
use crate::text::{TextContext, span::Text, layout::TextGalley};

/// A `zylo` rendering context.
///
/// Wraps a backend implementation.
pub struct Context<B> {
    backend: B,
    #[cfg(feature = "text")]
    text_context: TextContext,
}

impl<B> Context<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Self {
        #[cfg(feature = "text")]
        let text_context = {
            let mut cx = TextContext::new("Times New Roman".into());
            cx.fonts_mut().load_system_fonts();
            cx
        };
        Self {
            backend,
            #[cfg(feature = "text")]
            text_context,
        }
    }

    #[cfg(feature = "text")]
    pub fn load_font_from_data(&mut self, ttf_data: &[u8]) {
        self.text_context.fonts_mut().load_font_from_data(ttf_data)
    }

    #[cfg(feature = "text")]
    pub fn create_text_galley(&mut self, text: &Text) -> TextGalley {
        TextGalley::new(&mut self.text_context, text)
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
