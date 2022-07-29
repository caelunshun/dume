use std::fmt::Display;

#[cfg(feature = "color-to-linear")]
use fast_srgb8::{f32x4_to_srgb8, srgb8_to_f32};
#[cfg(feature = "color-to-linear")]
use glam::{vec4, Vec4};

/// An sRGB color with an alpha channel.
///
/// Unpremultiplied by convention.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color([u8; 4]);

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(u8::MAX, u8::MAX, u8::MAX);

    /// Creates a color from its RGBA components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    /// Creates a color from RGB components with 100% alpha.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, u8::MAX)
    }

    /// Gets the red component.
    pub fn red(&self) -> u8 {
        self.0[0]
    }

    /// Gets the green component.
    pub fn green(&self) -> u8 {
        self.0[1]
    }

    /// Gets the blue component.
    pub fn blue(&self) -> u8 {
        self.0[2]
    }

    /// Gets the alpha component.
    pub fn alpha(&self) -> u8 {
        self.0[3]
    }

    /// Gets the color as an array of values in RGBA order.
    pub fn to_array(&self) -> [u8; 4] {
        self.0
    }

    /// Creates a color from an array of values in RGBA order.
    pub fn from_array(array: [u8; 4]) -> Self {
        Self(array)
    }

    /// Encodes the color to linear RGB.
    ///
    /// Components are in the range `[0, 1]`.
    ///
    /// Linear RGB is appropriate for blending or otherwise
    /// operating on the color value.
    #[cfg(feature = "color-to-linear")]
    pub fn to_linear(&self) -> Vec4 {
        vec4(
            srgb8_to_f32(self.red()),
            srgb8_to_f32(self.green()),
            srgb8_to_f32(self.blue()),
            srgb8_to_f32(self.alpha()),
        )
    }

    /// Creates a color from linear RGB.
    ///
    /// The given components should lie within the range `[0, 1]`.
    #[cfg(feature = "color-to-linear")]
    pub fn from_linear(linear: Vec4) -> Self {
        let mut this = Self(f32x4_to_srgb8(linear.to_array()));
        this.0[3] = (linear.w * u8::MAX as f32).round() as u8;
        this
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rgba = self.to_array();
        write!(f, "#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])?;
        if rgba[3] != u8::MAX {
            write!(f, "{:02x}", rgba[3])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn hex_strings() {
        let color = Color::rgba(255, 254, 1, 255);
        assert_eq!(color.to_string(), "#fffe01");

        let color = Color::rgba(0, 0, 0, 128);
        assert_eq!(color.to_string(), "#00000080");
    }
}
