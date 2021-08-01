//
// Created by Caelum van Ispelen on 6/14/21.
//

#ifndef DUME_DUME_H
#define DUME_DUME_H

#include <dume_renderer.h>
#include <array>
#include <sol/sol.hpp>
#include <utility>

namespace dume {
    class Canvas;

    void makeLuaBindings(sol::state &lua);

    static sol::table convertMods(sol::state &lua, const Modifiers &mods) {
        return lua.create_table_with("control", mods.control, "alt", mods.alt, "shift", mods.shift);
    }

    static void invokeEvent(sol::function &luaEventCallback, sol::table event) {
        luaEventCallback.call<void>(event);
    }

    class Canvas {
        DumeCtx *ctx;
        Window *window;
        sol::table cursorPos;

    public:
        explicit Canvas(Window *window) : window(window) {
            ctx = dume_init(window);
        }

        ~Canvas() {
            dume_free(ctx);
            winit_window_free(window);
        }

        void resize(uint32_t newWidth, uint32_t newHeight, double newScaleFactor) {
            dume_resize(ctx, newWidth, newHeight, newScaleFactor);
        }

        uint64_t createSpriteFromEncoded(std::string name, std::string data) {
            return dume_create_sprite_from_encoded(ctx, reinterpret_cast<const uint8_t *>(name.data()), name.size(),
                                                   reinterpret_cast<const uint8_t *>(data.data()), data.size());
        }

        uint64_t createSpriteFromRGBA(std::string name, uint8_t *data, size_t dataSize, uint32_t width, uint32_t height) {
            return dume_create_sprite_from_rgba(ctx, reinterpret_cast<const uint8_t *>(name.data()), name.size(),
                                                   data, dataSize,
                                                   width, height);
        }

        uint64_t getSpriteByName(const std::string &name) {
            return dume_get_sprite_by_name(ctx, reinterpret_cast<const uint8_t *>(name.data()), name.size());
        }

        void loadFont(std::string fontData) {
            dume_load_font(ctx, reinterpret_cast<const uint8_t *>(fontData.data()), fontData.size());
        }

        Text *parseTextMarkup(const std::string &markup, CTextStyle defaultStyle, void *userdata, Variable(*resolveVariable) (void *, const uint8_t *, size_t)) {
            return dume_parse_markup(reinterpret_cast<const uint8_t *>(markup.data()), markup.size(), defaultStyle, userdata, resolveVariable);
        }

        Paragraph *createParagraph(Text *text, TextLayout layout) {
            return dume_create_paragraph(ctx, text, layout);
        }

        void drawParagraph(const Paragraph *paragraph, float x, float y) {
            dume_draw_paragraph(ctx, Vec2{ .x = x, .y = y }, paragraph);
        }

        void drawSprite(uint64_t sprite, float x, float y, float width) {
            dume_draw_sprite(ctx, Vec2 { .x = x, .y = y }, width, sprite);
        }

        void beginPath() {
            dume_begin_path(ctx);
        }

        void moveTo(float x, float y) {
            dume_move_to(ctx, Vec2 { .x = x, .y = y });
        }

        void lineTo(float x, float y) {
            dume_line_to(ctx, Vec2 { .x = x, .y = y });
        }

        void quadTo(float cx, float cy, float x, float y) {
            dume_quad_to(ctx, Vec2 { .x = cx, .y = cy }, Vec2 { .x = x, .y = y });
        }

        void cubicTo(float cx1, float cy1, float cx2, float cy2, float x, float y) {
            dume_cubic_to(ctx, Vec2 { .x = cx1, .y = cy1 }, Vec2 { .x = cx2, .y = cy2 }, Vec2 { .x = x, .y = y });
        }

        void arc(float cx, float cy, float radius, float startAngle, float endAngle) {
            dume_arc(ctx, Vec2 { .x = cx, .y = cy }, radius, startAngle, endAngle);
        }

        void strokeWidth(float width) {
            dume_stroke_width(ctx, width);
        }

        void solidColor(const uint8_t (*color)[4]) {
            dume_solid_color(ctx, color);
        }

        void linearGradient(float ax, float ay, float bx, float by, const uint8_t (*colorA)[4], const uint8_t (*colorB)[4]) {
            dume_linear_gradient(ctx, Vec2 { .x = ax, .y = ay }, Vec2 { .x = bx, .y = by }, colorA, colorB);
        }

        void stroke() {
            dume_stroke(ctx);
        }

        void fill() {
            dume_fill(ctx);
        }

        void resizeParagraph(Paragraph *paragraph, float newWidth, float newHeight) {
            dume_paragraph_resize(ctx, paragraph, Vec2 { .x = newWidth, .y = newHeight });
        }

        float getParagraphWidth(const Paragraph *paragraph) {
            return dume_paragraph_width(paragraph);
        }

        float getParagraphHeight(const Paragraph *paragraph) {
            return dume_paragraph_height(paragraph);
        }

        void translate(float x, float y) {
            dume_translate(ctx, Vec2 { .x = x, .y = y });
        }

        void scale(float scale) {
            dume_scale(ctx, scale);
        }

        void resetTransform() {
            dume_reset_transform(ctx);
        }

        void setScissorRect(float px, float py, float sx, float sy) {
            dume_scissor_rect(ctx, Vec2 { .x = px, .y = py }, Vec2 { .x = sx, .y = sy });
        }

        void clearScissor() {
            dume_clear_scissor(ctx);
        }

        Vec2 getSpriteSize(uint64_t id) {
            return dume_get_sprite_size(ctx, id);
        }

        uint32_t getWidth() {
            return dume_get_width(ctx);
        }

        uint32_t getHeight() {
            return dume_get_height(ctx);
        }

        void handleEvent(const Event &event, sol::state &lua, sol::function &luaEventHandler) {
            switch (event.kind) {
                case EventKind::MainEventsCleared:
                    winit_window_request_redraw(window);
                    break;
                case EventKind::Character:
                    invokeEvent(luaEventHandler, lua.create_table_with("type", "char", "char", event.data.c));
                    break;
                case EventKind::CursorMove:
                    cursorPos = lua.create_table_with("x", event.data.cursor_pos[0], "y", event.data.cursor_pos[1]);
                    invokeEvent(luaEventHandler, lua.create_table_with("type", "cursorMove", "pos", cursorPos));
                    break;
                case EventKind::Keyboard:
                    invokeEvent(luaEventHandler, lua.create_table_with("type", "key",
                                                                        "action", event.data.keyboard.action,
                                                                        "key", event.data.keyboard.key,
                                                                        "modifiers", convertMods(lua, event.data.keyboard.modifiers)
                                                                       ));
                    break;
                case EventKind::Mouse:
                    invokeEvent(luaEventHandler, lua.create_table_with(
                            "type", "mouseClick",
                            "action", event.data.mouse.action,
                            "mouse", event.data.mouse.mouse,
                            "pos", cursorPos,
                            "modifiers", convertMods(lua, event.data.mouse.modifiers)
                            ));
                    break;
                case EventKind::Scroll:
                    invokeEvent(luaEventHandler, lua.create_table_with(
                            "type", "scroll",
                            "offset", lua.create_table_with("x", event.data.scroll_delta[0], "y", event.data.scroll_delta[1]),
                            "pos", cursorPos
                            ));
                    break;
                case EventKind::Resized:
                    resize(event.data.new_size.dims[0], event.data.new_size.dims[1], event.data.new_size.scale_factor);
                    break;
                case EventKind::CloseRequested:
                case EventKind::RedrawRequested:
                    return;
            }
        }

        void render() {
            dume_render(ctx);
        }
    };
}

#endif //DUME_DUME_H
