//
// Created by Caelum van Ispelen on 6/14/21.
//

#ifndef DUME_DUME_H
#define DUME_DUME_H

#define GLFW_EXPOSE_NATIVE_X11

#include <dume_renderer.h>
#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>
#include <array>
#include <sol/sol.hpp>
#include <utility>

namespace dume {
    class Canvas;

    void makeLuaBindings(sol::state &lua);

    static std::unique_ptr<sol::function> luaEventCallback;
    static std::unique_ptr<sol::function> luaResizeCallback;
    static std::shared_ptr<sol::state> lua;
    static std::shared_ptr<Canvas> cv;

    static sol::table convertMods(int mods) {
        bool control = mods & GLFW_MOD_CONTROL;
        bool alt = mods & GLFW_MOD_ALT;
        bool shift = mods & GLFW_MOD_SHIFT;

        return lua->create_table_with("control", control, "alt", alt, "shift", shift);
    }

    static void invokeEvent(sol::table event) {
        luaEventCallback->call<void>(event);
    }

    static void resizeCallback(GLFWwindow *window, int width, int height);

    static void keyCallback(GLFWwindow* window, int key, int scancode, int action, int mods) {
        auto table = lua->create_table_with(
                    "type", "key",
                    "action", action,
                    "key", key,
                    "modifiers", convertMods(mods)
                );
        invokeEvent(table);
    }

    static void charCallback(GLFWwindow* window, unsigned int codepoint) {
        auto table = lua->create_table_with(
                "type", "char",
                "char", codepoint
                );
        invokeEvent(table);
    }

    static void cursorPositionCallback(GLFWwindow* window, double xpos, double ypos) {
        auto table = lua->create_table_with(
                "type", "cursorMove",
                "pos", lua->create_table_with("x", xpos, "y", ypos)
                );
        invokeEvent(table);
    }

    static void mousePressCallback(GLFWwindow* window, int mouse, int action, int mods) {
        double xpos, ypos;
        glfwGetCursorPos(window, &xpos, &ypos);
        auto table = lua->create_table_with(
                "type", "mouseClick",
                "mouse", mouse,
                "action", action,
                "modifiers", convertMods(mods),
                "pos", lua->create_table_with("x", xpos, "y", ypos)
                );
        invokeEvent(table);
    }

    static void scrollCallback(GLFWwindow* window, double xoffset, double yoffset) {
        double xpos, ypos;
        glfwGetCursorPos(window, &xpos, &ypos);
        auto table = lua->create_table_with(
                "type", "scroll",
                "offset", lua->create_table_with("x", xoffset, "y", yoffset),
                "pos", lua->create_table_with("x", xpos, "y", ypos)
                );
        invokeEvent(table);
    }

    class Canvas {
        DumeCtx *ctx;
        GLFWwindow *window;

    public:
        explicit Canvas(GLFWwindow *window) : window(window) {
            int width, height;
            glfwGetWindowSize(window, &width, &height);
            ctx = dume_init(width, height, RawWindow {
                .window = glfwGetX11Window(window),
                .display = glfwGetX11Display()
            });
        }

        ~Canvas() {
            dume_free(ctx);
        }

        void resize(uint32_t newWidth, uint32_t newHeight) {
            dume_resize(ctx, newWidth, newHeight);
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

        void setGlfwCallbacks(std::shared_ptr<Canvas> self, std::shared_ptr<sol::state> lua_, sol::function eventCallback, sol::function resizeCallbackFn) {
            luaEventCallback = std::make_unique<sol::function>(eventCallback);
            luaResizeCallback = std::make_unique<sol::function>(resizeCallbackFn);
            lua = std::move(lua_);
            cv = std::move(self);

            glfwSetWindowSizeCallback(window, resizeCallback);
            glfwSetScrollCallback(window, scrollCallback);
            glfwSetKeyCallback(window, keyCallback);
            glfwSetCharCallback(window, charCallback);
            glfwSetMouseButtonCallback(window, mousePressCallback);
            glfwSetCursorPosCallback(window, cursorPositionCallback);
        }

        void render() {
            dume_render(ctx);
        }
    };

    void resizeCallback(GLFWwindow *window, int width, int height) {
        luaResizeCallback->call<void>(lua->create_table_with("x", width, "y", height));
        cv->resize(width, height);
    }
}

#endif //DUME_DUME_H
