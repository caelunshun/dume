# dume
A 2D rendering canvas for `wgpu`, largely inspired by the HTML5 Canvas API.

## Features
* Path and sprite rendering with linear gradients, radial gradients, fills, and strokes
* Text rendering API with rich (multicolored) text, subpixel antialiasing, and shaping/kerning based on [`rustybuzz`](https://github.com/RazrFalcon/rustybuzz)
* Support for rendering YUV textures (useful for rendering video frames, which are stored in YUV formats in many codecs)

The development of this library was mainly driven by [`riposte`](https://github.com/caelunshun/riposte).
