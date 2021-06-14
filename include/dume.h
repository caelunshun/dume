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

        void drawSprite(uint64_t sprite, float x, float y, float width) {
            dume_draw_sprite(ctx, Vec2 { .x = x, .y = y }, width, sprite);
        }

        void render() {
            dume_render(ctx);
        }
    };
}

#endif //DUME_DUME_H
