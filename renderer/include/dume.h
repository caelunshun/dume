#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>
#include "glam.h"


static const uint32_t SAMPLE_COUNT = 8;

enum class Align {
  /// Top or left
  Start,
  /// Middle or center
  Center,
  /// Bottom or right
  End,
};

/// Defines the baseline of a line of text.
enum class Baseline {
  Top,
  Middle,
  Alphabetic,
  Bottom,
};

struct DumeCtx;

/// A paragraph of rich text that has been layed
/// out and is ready for rendering.
struct Paragraph;

/// Some rich text. Implemented as a list of [`TextSection`]s.
struct Text;

/// Settings for laying out text.
///
/// TODO: should some parameters be moved to the rich text
/// representation, so that alignments can be mixed within a paragraph?
struct TextLayout {
  /// The maximum dimensions of the formatted text.
  ///
  /// Excess text is hidden.
  Vec2 max_dimensions;
  /// Whether to overflow onto a new line when the maximum width is reached.
  ///
  /// If false, then excess characters are omitted.
  bool line_breaks;
  /// The baseline to use.
  Baseline baseline;
  /// Horizontal alignment to apply to the text.
  Align align_h;
  /// Vertical alignment to apply to the text.
  Align align_v;
};

struct RawWindow {
  unsigned long window;
  void *display;
};

struct Variable {
  const uint8_t *value;
  size_t len;
};


extern "C" {

/// NB: consumes the text.
Paragraph *dume_create_paragraph(DumeCtx *ctx, Text *text, TextLayout layout);

uint64_t dume_create_sprite_from_encoded(DumeCtx *ctx,
                                         const uint8_t *name,
                                         size_t name_len,
                                         const uint8_t *data,
                                         size_t data_len);

uint64_t dume_create_sprite_from_rgba(DumeCtx *ctx,
                                      const uint8_t *name,
                                      size_t name_len,
                                      uint8_t *data,
                                      size_t data_len,
                                      uint32_t width,
                                      uint32_t height);

void dume_draw_paragraph(DumeCtx *ctx, Vec2 pos, const Paragraph *paragraph);

void dume_draw_sprite(DumeCtx *ctx, Vec2 pos, float width, uint64_t sprite);

DumeCtx *dume_init(uint32_t width, uint32_t height, RawWindow window);

void dume_load_font(DumeCtx *ctx, const uint8_t *font_data, size_t font_len);

void dume_paragraph_free(Paragraph *paragraph);

void dume_paragraph_resize(DumeCtx *ctx, Paragraph *paragraph, Vec2 new_max_dimensions);

Text *dume_parse_markup(const uint8_t *markup,
                        size_t markup_len,
                        void *userdata,
                        Variable (*resolve_variable)(void*, const uint8_t*, size_t));

void dume_render(DumeCtx *ctx);

void dume_resize(DumeCtx *ctx, uint32_t new_width, uint32_t new_height);

void dume_text_free(Text *text);

} // extern "C"
