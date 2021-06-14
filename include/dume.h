//
// Created by Caelum van Ispelen on 6/14/21.
//

#ifndef DUME_DUME_H
#define DUME_DUME_H

#define GLFW_EXPOSE_NATIVE_X11

#include <dume_renderer.h>
#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>

namespace dume {
    static Variable resolveDefaultVariable(void *userdata, const uint8_t *name, size_t len) {
        return Variable{.value = nullptr, .len=0};
    }

    class Canvas {
        DumeCtx *ctx;

    public:
        explicit Canvas(GLFWwindow *window) {
            int width, height;
            glfwGetWindowSize(window, &width, &height);
            ctx = dume_init(width, height, RawWindow {
                .window = glfwGetX11Window(window),
                .display = glfwGetX11Display()
            });
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

        void loadFont(std::string fontData) {
            dume_load_font(ctx, reinterpret_cast<const uint8_t *>(fontData.data()), fontData.size());
        }

        Text *parseTextMarkup(const std::string &markup, void *userdata, Variable(*resolveVariable) (void *, const uint8_t *, size_t)) {
            return dume_parse_markup(reinterpret_cast<const uint8_t *>(markup.data()), markup.size(), userdata, resolveVariable);
        }

        Text *parseTextMarkupDefault(const std::string &markup) {
            return parseTextMarkup(markup, nullptr, resolveDefaultVariable);
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

        void render() {
            dume_render(ctx);
        }
    };
}

#endif //DUME_DUME_H
