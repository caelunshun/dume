#include <dume.h>
#include <vector>

int main() {
    glfwInit();
    auto *window = glfwCreateWindow(1920 / 2, 1080 / 2, "Dume", nullptr, nullptr);

    dume::Canvas canvas(window);

    std::vector<uint8_t> rgba(128 * 128 * 4);
    for (int x = 0; x < 128; x++) {
        for (int y = 0; y < 128; y++) {
            rgba[(x + y * 128) * 4] = x * 2;
            rgba[(x + y * 128) * 4 + 1] = x * 2;
            rgba[(x + y * 128) * 4 + 2] = x * 2;
            rgba[(x + y * 128) * 4 + 3] = 255;
        }
    }
    auto sprite = canvas.createSpriteFromRGBA("sprite", rgba.data(), rgba.size(), 128, 128);

    while (!glfwWindowShouldClose(window)) {
        canvas.drawSprite(sprite, 30, 30, 600);
        canvas.render();

        glfwPollEvents();
    }

    return 0;
}
