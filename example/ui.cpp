#include <dume.h>
#include <vector>
#include <fstream>
#include <iostream>

std::string loadFile(const std::string &path) {
    std::ifstream t(path);
    std::string str((std::istreambuf_iterator<char>(t)),
                    std::istreambuf_iterator<char>());
    return str;
}

int main() {
    glfwInit();
    auto *window = glfwCreateWindow(1920 / 2, 1080 / 2, "Dume", nullptr, nullptr);

    auto canvas = std::make_shared<dume::Canvas>(window);

    std::vector<uint8_t> rgba(128 * 128 * 4);
    for (int x = 0; x < 128; x++) {
        for (int y = 0; y < 128; y++) {
            rgba[(x + y * 128) * 4] = x;
            rgba[(x + y * 128) * 4 + 1] = x;
            rgba[(x + y * 128) * 4 + 2] = x;
            rgba[(x + y * 128) * 4 + 3] = 255;
        }
    }
    canvas->createSpriteFromRGBA("gradient", rgba.data(), rgba.size(), 128, 128);

    canvas->createSpriteFromEncoded("smoke", loadFile("/home/caelum/Pictures/volume1.png"));

    canvas->loadFont(loadFile("/home/caelum/Downloads/Merriweather-Regular.ttf"));
    canvas->loadFont(loadFile("/home/caelum/Downloads/Merriweather-Italic.ttf"));
    canvas->loadFont(loadFile("/home/caelum/Downloads/Merriweather-Bold.ttf"));
    canvas->loadFont(loadFile("/home/caelum/Downloads/Merriweather-BoldItalic.ttf"));

    auto lua = std::make_shared<sol::state>();
    lua->open_libraries(sol::lib::base, sol::lib::package, sol::lib::string, sol::lib::math, sol::lib::table, sol::lib::os);
    dume::makeLuaBindings(*lua);
    (*lua)["cv"] = canvas;
    lua->script(loadFile("example/draw.lua"));

    sol::function drawFunction = (*lua)["draw"];
    sol::function eventFunction = (*lua)["handleEvent"];
    sol::function resizeFunction = (*lua)["resize"];

    canvas->setGlfwCallbacks(canvas, lua, eventFunction, resizeFunction);

    while (!glfwWindowShouldClose(window)) {
        drawFunction.call<void>();

        canvas->render();
        glfwPollEvents();
    }

    return 0;
}
