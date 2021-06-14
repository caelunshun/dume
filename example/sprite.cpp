#include <dume.h>
#include <vector>
#include <fstream>

std::string loadFile(const std::string &path) {
    std::ifstream t(path);
    std::string str((std::istreambuf_iterator<char>(t)),
                    std::istreambuf_iterator<char>());
    return str;
}

int main() {
    glfwInit();
    auto *window = glfwCreateWindow(1920 / 2, 1080 / 2, "Dume", nullptr, nullptr);

    dume::Canvas canvas(window);

    std::vector<uint8_t> rgba(128 * 128 * 4);
    for (int x = 0; x < 128; x++) {
        for (int y = 0; y < 128; y++) {
            rgba[(x + y * 128) * 4] = x;
            rgba[(x + y * 128) * 4 + 1] = x;
            rgba[(x + y * 128) * 4 + 2] = x;
            rgba[(x + y * 128) * 4 + 3] = 255;
        }
    }
    auto sprite = canvas.createSpriteFromRGBA("sprite", rgba.data(), rgba.size(), 128, 128);

    canvas.loadFont(loadFile("/home/caelum/Downloads/Merriweather-Regular.ttf"));
    canvas.loadFont(loadFile("/home/caelum/Downloads/Merriweather-Italic.ttf"));
    canvas.loadFont(loadFile("/home/caelum/Downloads/Merriweather-Bold.ttf"));
    canvas.loadFont(loadFile("/home/caelum/Downloads/Merriweather-BoldItalic.ttf"));

    auto text = canvas.parseTextMarkupDefault("@size{30}{I am @bold{Dume}. @italic{I am the Bendu.}}");
    auto paragraph = canvas.createParagraph(text, TextLayout {
            .max_dimensions = Vec2 {
                    .x = 1920 / 2,
                    .y = 1080 / 2,
            },
            .line_breaks = true,
            .baseline = Baseline::Top,
         .align_h = Align::Center,
         .align_v = Align::Center,
    });

    while (!glfwWindowShouldClose(window)) {
        canvas.drawSprite(sprite, 30, 30, 600);
        canvas.drawParagraph(paragraph, 0, 0);

        canvas.render();
        glfwPollEvents();
    }

    return 0;
}
