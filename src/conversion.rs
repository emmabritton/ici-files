use crate::color::Color;

#[inline(always)]
fn f32_to_u8(value: f32) -> u8 {
    (value * 255.).round().clamp(0., 255.) as u8
}

/// Converts to/from RGB
pub trait OpaqueColorConversion<T> {
    fn to_rgb(self) -> T;
    /// Sets alpha to 255
    fn from_rgb(value: T) -> Color;
}

/// Converts to/from RGBA and ARGB
pub trait ColorConversion<T> {
    fn to_rgba(self) -> T;
    fn from_rgba(value: T) -> Color;
    fn to_argb(self) -> T;
    fn from_argb(value: T) -> Color;
}

impl OpaqueColorConversion<[u8; 3]> for Color {
    #[inline]
    fn to_rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    #[inline]
    fn from_rgb(value: [u8; 3]) -> Color {
        Color::new(value[0], value[1], value[2], 255)
    }
}

impl OpaqueColorConversion<(u8, u8, u8)> for Color {
    #[inline]
    fn to_rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    #[inline]
    fn from_rgb(value: (u8, u8, u8)) -> Color {
        Color::new(value.0, value.1, value.2, 255)
    }
}

impl OpaqueColorConversion<[f32; 3]> for Color {
    #[inline]
    fn to_rgb(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    #[inline]
    fn from_rgb(value: [f32; 3]) -> Color {
        Color::new(
            f32_to_u8(value[0]),
            f32_to_u8(value[1]),
            f32_to_u8(value[2]),
            255,
        )
    }
}

impl OpaqueColorConversion<(f32, f32, f32)> for Color {
    #[inline]
    fn to_rgb(self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    #[inline]
    fn from_rgb(value: (f32, f32, f32)) -> Color {
        Color::new(
            f32_to_u8(value.0),
            f32_to_u8(value.1),
            f32_to_u8(value.2),
            255,
        )
    }
}

impl ColorConversion<[u8; 4]> for Color {
    #[inline]
    fn to_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    #[inline]
    fn to_argb(self) -> [u8; 4] {
        [self.a, self.r, self.g, self.b]
    }

    #[inline]
    fn from_rgba(value: [u8; 4]) -> Color {
        Color::new(value[0], value[1], value[2], value[3])
    }

    #[inline]
    fn from_argb(value: [u8; 4]) -> Color {
        Color::new(value[1], value[2], value[3], value[0])
    }
}

impl ColorConversion<(u8, u8, u8, u8)> for Color {
    #[inline]
    fn to_rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    #[inline]
    fn to_argb(self) -> (u8, u8, u8, u8) {
        (self.a, self.r, self.g, self.b)
    }

    #[inline]
    fn from_rgba(value: (u8, u8, u8, u8)) -> Color {
        Color::new(value.0, value.1, value.2, value.3)
    }

    #[inline]
    fn from_argb(value: (u8, u8, u8, u8)) -> Color {
        Color::new(value.1, value.2, value.3, value.0)
    }
}

impl ColorConversion<u32> for Color {
    #[inline]
    fn to_rgba(self) -> u32 {
        u32::from_be_bytes([self.r, self.g, self.b, self.a])
    }

    #[inline]
    fn from_rgba(value: u32) -> Color {
        let bytes = value.to_be_bytes();
        Color::new(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    #[inline]
    fn to_argb(self) -> u32 {
        u32::from_be_bytes([self.a, self.r, self.g, self.b])
    }

    #[inline]
    fn from_argb(value: u32) -> Color {
        let bytes = value.to_be_bytes();
        Color::new(bytes[1], bytes[2], bytes[3], bytes[0])
    }
}

impl ColorConversion<[f32; 4]> for Color {
    #[inline]
    fn to_rgba(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    #[inline]
    fn from_rgba(value: [f32; 4]) -> Color {
        Color::new(
            f32_to_u8(value[0]),
            f32_to_u8(value[1]),
            f32_to_u8(value[2]),
            f32_to_u8(value[3]),
        )
    }

    #[inline]
    fn to_argb(self) -> [f32; 4] {
        [
            self.a as f32 / 255.0,
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    #[inline]
    fn from_argb(value: [f32; 4]) -> Color {
        Color::new(
            f32_to_u8(value[1]),
            f32_to_u8(value[2]),
            f32_to_u8(value[3]),
            f32_to_u8(value[0]),
        )
    }
}

impl ColorConversion<(f32, f32, f32, f32)> for Color {
    #[inline]
    fn to_rgba(self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    #[inline]
    fn from_rgba(value: (f32, f32, f32, f32)) -> Color {
        Color::new(
            f32_to_u8(value.0),
            f32_to_u8(value.1),
            f32_to_u8(value.2),
            f32_to_u8(value.3),
        )
    }

    #[inline]
    fn to_argb(self) -> (f32, f32, f32, f32) {
        (
            self.a as f32 / 255.0,
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    #[inline]
    fn from_argb(value: (f32, f32, f32, f32)) -> Color {
        Color::new(
            f32_to_u8(value.1),
            f32_to_u8(value.2),
            f32_to_u8(value.3),
            f32_to_u8(value.0),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn check_color_conversion_u32() {
        let red = RED;
        let argb: u32 = red.to_argb();
        let rgba: u32 = red.to_rgba();
        assert_eq!(red, Color::from_argb(argb));
        assert_eq!(red, Color::from_rgba(rgba));
    }

    #[test]
    fn check_color_conversion_f32_arr() {
        let red = RED;
        let argb: [f32; 4] = red.to_argb();
        let rgba: [f32; 4] = red.to_rgba();
        assert_eq!(red, Color::from_argb(argb));
        assert_eq!(red, Color::from_rgba(rgba));
    }

    #[test]
    fn check_color_conversion_f32_tuple() {
        let red = RED;
        let argb: (f32, f32, f32, f32) = red.to_argb();
        let rgba: (f32, f32, f32, f32) = red.to_rgba();
        assert_eq!(red, Color::from_argb(argb));
        assert_eq!(red, Color::from_rgba(rgba));
    }

    #[test]
    fn check_color_conversion_u8_arr() {
        let red = RED;
        let argb: [u8; 4] = red.to_argb();
        let rgba: [u8; 4] = red.to_rgba();
        assert_eq!(red, Color::from_argb(argb));
        assert_eq!(red, Color::from_rgba(rgba));
    }

    #[test]
    fn check_color_conversion_u8_tuple() {
        let red = RED;
        let argb: (u8, u8, u8, u8) = red.to_argb();
        let rgba: (u8, u8, u8, u8) = red.to_rgba();
        assert_eq!(red, Color::from_argb(argb));
        assert_eq!(red, Color::from_rgba(rgba));
    }
}
