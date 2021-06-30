#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>
#include "glam.h"


static const uint32_t SAMPLE_COUNT = 8;

enum class Action {
  Press,
  Release,
};

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

enum class CControlFlow {
  Poll,
  Exit,
};

enum class EventKind {
  CloseRequested,
  RedrawRequested,
  MainEventsCleared,
  Resized,
  Character,
  Keyboard,
  Mouse,
  CursorMove,
  Scroll,
};

/// Font style: normal or italic. We do not support
/// oblique fonts.
enum class Style {
  Normal,
  Italic,
};

/// A font weight, indicating how dark it appears.
enum class Weight {
  Thin,
  ExtraLight,
  Light,
  Normal,
  Medium,
  SemiBold,
  Bold,
  ExtraBold,
  Black,
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

struct CTextStyle {
  const char *family_name;
  size_t family_name_len;
  Weight weight;
  Style style;
  float size;
  const uint8_t *color;
};

struct Variable {
  const uint8_t *value;
  size_t len;
};

struct Modifiers {
  bool control;
  bool alt;
  bool shift;
};

struct KeyboardEvent {
  uint32_t key;
  Action action;
  Modifiers modifiers;
};

struct MouseEvent {
  uint32_t mouse;
  Action action;
  Modifiers modifiers;
};

union EventData {
  uint8_t empty;
  uint32_t new_size[2];
  uint32_t c;
  KeyboardEvent keyboard;
  MouseEvent mouse;
  float cursor_pos[2];
  float scroll_delta[2];
};

struct Event {
  EventKind kind;
  EventData data;
};

struct WindowOptions {
  const char *title;
  uint32_t width;
  uint32_t height;
};


extern "C" {

void dume_arc(DumeCtx *ctx, Vec2 center, float radius, float start_angle, float end_angle);

void dume_begin_path(DumeCtx *ctx);

void dume_clear_scissor(DumeCtx *ctx);

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

void dume_cubic_to(DumeCtx *ctx, Vec2 control1, Vec2 control2, Vec2 pos);

void dume_draw_paragraph(DumeCtx *ctx, Vec2 pos, const Paragraph *paragraph);

void dume_draw_sprite(DumeCtx *ctx, Vec2 pos, float width, uint64_t sprite);

void dume_fill(DumeCtx *ctx);

void dume_free(DumeCtx *ctx);

uint32_t dume_get_height(DumeCtx *ctx);

uint64_t dume_get_sprite_by_name(DumeCtx *ctx, const uint8_t *name, size_t name_len);

Vec2 dume_get_sprite_size(DumeCtx *ctx, uint64_t sprite);

uint32_t dume_get_width(DumeCtx *ctx);

DumeCtx *dume_init(const Window *window);

void dume_line_to(DumeCtx *ctx, Vec2 pos);

void dume_linear_gradient(DumeCtx *ctx,
                          Vec2 point_a,
                          Vec2 point_b,
                          const uint8_t (*color_a)[4],
                          const uint8_t (*color_b)[4]);

void dume_load_font(DumeCtx *ctx, const uint8_t *font_data, size_t font_len);

void dume_move_to(DumeCtx *ctx, Vec2 pos);

void dume_paragraph_free(Paragraph *paragraph);

float dume_paragraph_height(const Paragraph *p);

void dume_paragraph_resize(DumeCtx *ctx, Paragraph *paragraph, Vec2 new_max_dimensions);

float dume_paragraph_width(const Paragraph *p);

Text *dume_parse_markup(const uint8_t *markup,
                        size_t markup_len,
                        CTextStyle default_style,
                        void *userdata,
                        Variable (*resolve_variable)(void*, const uint8_t*, size_t));

void dume_quad_to(DumeCtx *ctx, Vec2 control, Vec2 pos);

void dume_render(DumeCtx *ctx);

void dume_reset_transform(DumeCtx *ctx);

void dume_resize(DumeCtx *ctx, uint32_t new_width, uint32_t new_height);

void dume_scale(DumeCtx *ctx, float scale);

void dume_scissor_rect(DumeCtx *ctx, Vec2 pos, Vec2 size);

void dume_solid_color(DumeCtx *ctx, const uint8_t (*color)[4]);

void dume_stroke(DumeCtx *ctx);

void dume_stroke_width(DumeCtx *ctx, float width);

void dume_text_free(Text *text);

void dume_translate(DumeCtx *ctx, Vec2 vector);

EventLoop *winit_event_loop_new();

void winit_event_loop_run(EventLoop *event_loop,
                          CControlFlow (*callback)(void*, Event),
                          void *userdata);

double winit_get_time();

void winit_window_free(Window *window);

void winit_window_grab_cursor(const Window *window, bool grabbed);

Window *winit_window_new(const WindowOptions *options, const EventLoop *event_loop);

void winit_window_request_redraw(const Window *window);

void winit_window_set_cursor_pos(const Window *window, float x, float y);

} // extern "C"
