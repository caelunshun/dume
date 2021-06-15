//
// Created by Caelum van Ispelen on 6/14/21.
//

#include <dume.h>

namespace dume {
    static Variable luaResolveVariable(void *variables, const uint8_t *var, size_t len) {
        sol::table &vars = *((sol::table*) variables);
        std::string varName(reinterpret_cast<const char*>(var), len);

        const std::string &value = vars[varName];
        uint8_t *data = static_cast<uint8_t *>(malloc(value.size()));
        memcpy(data, value.data(), value.size());
        return Variable {
            .value = data,
            .len = value.size(),
        };
    }

    void makeLuaBindings(sol::state &lua) {
        auto canvas_type = lua.new_usertype<Canvas>("Canvas");
        canvas_type["drawSprite"] = [](Canvas &self, const std::string &name, sol::table pos, float width) {
            auto sprite = self.getSpriteByName(name);
            self.drawSprite(sprite, pos["x"], pos["y"], width);
        };
        canvas_type["beginPath"] = &Canvas::beginPath;
        canvas_type["moveTo"] = [](Canvas &self, sol::table pos) {
            self.moveTo(pos["x"], pos["y"]);
        };
        canvas_type["lineTo"] = [](Canvas &self, sol::table pos) {
            self.lineTo(pos["x"], pos["y"]);
        };
        canvas_type["quadTo"] = [](Canvas &self, sol::table control, sol::table pos) {
            self.quadTo(control["x"], control["y"], pos["x"], pos["y"]);
        };
        canvas_type["cubicTo"] = [](Canvas &self, sol::table control1, sol::table control2, sol::table pos) {
            self.cubicTo(control1["x"], control1["y"], control2["x"], control2["y"], pos["x"], pos["y"]);
        };
        canvas_type["strokeWidth"] = &Canvas::strokeWidth;
        canvas_type["stroke"] = &Canvas::stroke;
        canvas_type["fill"] = &Canvas::fill;
        canvas_type["solidColor"] = [](Canvas &self, sol::table color) {
            uint8_t col[4] = {color[1], color[2], color[3], color[4]};
            self.solidColor(&col);
        };
        canvas_type["linearGradient"] = [](Canvas &self, sol::table pointA, sol::table pointB, sol::table colorA, sol::table colorB) {
            uint8_t colA[4] = {colorA[1], colorA[2], colorA[3], colorA[4]};
            uint8_t colB[4] = {colorB[1], colorB[2], colorB[3], colorB[4]};
            self.linearGradient(pointA["x"], pointA["y"], pointB["x"], pointB["y"], &colA, &colB);
        };

        canvas_type["parseTextMarkup"] = [](Canvas &self, std::string markup, sol::table variables) {
            return sol::light<Text>(self.parseTextMarkup(markup, &variables, luaResolveVariable));
        };

        canvas_type["createParagraph"] = [](Canvas &self, sol::light<Text> text, sol::table layout) {
            const auto lay = TextLayout {
                .max_dimensions = Vec2 { .x = layout["maxDimensions"]["x"], .y = layout["maxDimensions"]["y"] },
                .line_breaks = layout["lineBreaks"],
                .baseline = layout["baseline"],
                .align_h = layout["alignH"],
                .align_v = layout["alignV"],
            };
            return sol::light<Paragraph>(self.createParagraph(text, lay));
        };

        canvas_type["drawParagraph"] = [](Canvas &self, sol::light<Paragraph> paragraph, sol::table pos) {
            self.drawParagraph(paragraph, pos["x"], pos["y"]);
        };

        canvas_type["resizeParagraph"] = [](Canvas &self, sol::light<Paragraph> paragraph, sol::table newSize) {
            self.resizeParagraph(paragraph, newSize["x"], newSize["y"]);
        };

        canvas_type["getParagraphWidth"] = [](Canvas &self, sol::light<Paragraph> p) {
            return self.getParagraphWidth(p);
        };
        canvas_type["getParagraphHeight"] = [](Canvas &self, sol::light<Paragraph> p) {
            return self.getParagraphHeight(p);
        };

        canvas_type["translate"] = [](Canvas &self, sol::table vector) {
            self.translate(vector["x"], vector["y"]);
        };

        canvas_type["scale"] = &Canvas::scale;

        canvas_type["resetTransform"] = &Canvas::resetTransform;

        canvas_type["getSpriteSize"] = [](Canvas &self, const std::string sprite, sol::table target) {
            auto size = self.getSpriteSize(self.getSpriteByName(sprite));
            target["x"] = size.x;
            target["y"] = size.y;
        };

        canvas_type["getWidth"] = &Canvas::getWidth;
        canvas_type["getHeight"] = &Canvas::getHeight;
    }
}
