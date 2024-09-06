use crate::Tint;

use crate::errors::IndexedImageError;
use crate::prelude::IndexedImageError::InvalidHexFormat;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

///This represents an RGBA color
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    //red channel
    pub r: u8,
    //green channel
    pub g: u8,
    //blue channel
    pub b: u8,
    //alpha channel
    pub a: u8,
}

#[cfg(feature = "serde")]
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u32(self.into())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let num = u32::deserialize(d)?;
        Ok(num.into())
    }
}

impl Color {
    pub const fn with_red(&self, red: u8) -> Color {
        Color {
            r: red,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    pub const fn with_green(&self, green: u8) -> Color {
        Color {
            r: self.r,
            g: green,
            b: self.b,
            a: self.a,
        }
    }

    pub const fn with_blue(&self, blue: u8) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: blue,
            a: self.a,
        }
    }

    pub const fn with_alpha(&self, alpha: u8) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}

#[inline]
fn f32_to_u8(value: f32) -> u8 {
    (value * 255.).round().clamp(0., 255.) as u8
}

impl Default for Color {
    fn default() -> Self {
        Color::new(0, 0, 0, 255)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Color::new(value.0, value.1, value.2, value.3)
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Color::new(value[0], value[1], value[2], value[3])
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Color::new(value.0, value.1, value.2, 255)
    }
}

impl From<[u8; 3]> for Color {
    fn from(value: [u8; 3]) -> Self {
        Color::new(value[0], value[1], value[2], 255)
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        let bytes = value.to_be_bytes();
        Color::new(bytes[0], bytes[1], bytes[2], bytes[3])
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        u32::from_be_bytes(value.as_array())
    }
}

impl From<&Color> for u32 {
    fn from(value: &Color) -> Self {
        u32::from_be_bytes(value.as_array())
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Color::new(
            f32_to_u8(value.0),
            f32_to_u8(value.1),
            f32_to_u8(value.2),
            f32_to_u8(value.3),
        )
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Color::new(
            f32_to_u8(value[0]),
            f32_to_u8(value[1]),
            f32_to_u8(value[2]),
            f32_to_u8(value[3]),
        )
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32)) -> Self {
        Color::new(
            f32_to_u8(value.0),
            f32_to_u8(value.1),
            f32_to_u8(value.2),
            255,
        )
    }
}

impl From<[f32; 3]> for Color {
    fn from(value: [f32; 3]) -> Self {
        Color::new(
            f32_to_u8(value[0]),
            f32_to_u8(value[1]),
            f32_to_u8(value[2]),
            255,
        )
    }
}

impl Color {
    /// Converts 0.0..=1.0 to 0..=255
    /// Values outside 0.0..=1.0 are clamped
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new color with `red`, `green` and `blue` set to `value` and `alpha` set to 255
    #[inline]
    pub const fn gray(value: u8) -> Self {
        Color::new(value, value, value, 255)
    }

    /// Convert an i32 into a [`Color`] where bytes match the format [R,G,B,A]
    #[inline]
    pub const fn from_i32(value: i32) -> Self {
        let bytes = value.to_be_bytes();
        Color::new(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    pub fn from_hex(hex: &str) -> Result<Color, IndexedImageError> {
        let mut hex = hex.to_string();
        if hex.starts_with('#') {
            hex.remove(0);
        }
        if hex.chars().count() != 6 && hex.chars().count() != 8 {
            return Err(InvalidHexFormat("wrong length".to_string()));
        }
        if hex.chars().any(|c| !c.is_ascii_hexdigit()) {
            return Err(InvalidHexFormat("non hex digits".to_string()));
        }
        let chars: Vec<char> = hex.chars().collect();
        let mut colours = vec![];
        for digits in chars.chunks_exact(2) {
            let num = u8::from_str_radix(&format!("{}{}", digits[0], digits[1]), 16)
                .map_err(|e| InvalidHexFormat(e.to_string()))?;
            colours.push(num);
        }
        if colours.len() == 3 {
            colours.push(255);
        }
        Ok(Color {
            r: colours[0],
            g: colours[1],
            b: colours[2],
            a: colours[3],
        })
    }
}

impl Color {
    /// Split color into array in the format [R,G,B,A]
    #[inline]
    pub fn as_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert color to i32 in the format [R,G,B,A]
    #[inline]
    pub fn as_i32(&self) -> i32 {
        i32::from_be_bytes(self.as_array())
    }

    /// Convert color to f32 array in the format [R,G,B,A] where 0.0 = 0, and 1.0 = 255
    #[inline]
    pub fn as_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn blend(&self, other: Color) -> Color {
        let base = self.as_f32_array();
        let added = other.as_f32_array();
        let mut mix = [0.0, 0.0, 0.0, 0.0];
        mix[3] = 1.0 - (1.0 - added[3]) * (1.0 - base[3]);
        mix[0] = (added[0] * added[3] / mix[3]) + (base[0] * base[3] * (1.0 - added[3]) / mix[3]);
        mix[1] = (added[1] * added[3] / mix[3]) + (base[1] * base[3] * (1.0 - added[3]) / mix[3]);
        mix[2] = (added[2] * added[3] / mix[3]) + (base[2] * base[3] * (1.0 - added[3]) / mix[3]);

        mix.into()
    }

    /// ignores alpha
    pub fn brightness(&self) -> f32 {
        let new = self.as_f32_array();
        0.2126 * new[0] + 0.7152 * new[1] + 0.0722 * new[2]
    }

    #[inline]
    pub fn is_dark(&self) -> bool {
        self.brightness() < 0.5
    }

    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.a < 255
    }

    pub fn darken(&self) -> Color {
        self.with_brightness(0.9)
    }

    pub fn lighten(&self) -> Color {
        self.with_brightness(1.1)
    }

    /// Copy color with brightness
    pub fn with_brightness(&self, amount: f32) -> Color {
        let new = self.as_f32_array();
        (
            (new[0] * amount).min(1.0).max(0.0),
            (new[1] * amount).min(1.0).max(0.0),
            (new[2] * amount).min(1.0).max(0.0),
            new[3],
        )
            .into()
    }

    /// De/saturate color by percentage
    /// Negative amount increases saturation
    pub fn with_saturate(&self, amount: f32) -> Color {
        let mut new = self.as_f32_array();
        let lum = 0.2989 * new[0] + 0.5870 * new[1] + 0.1140 * new[2];
        new[0] = new[0] + amount * (lum - new[0]);
        new[1] = new[1] + amount * (lum - new[1]);
        new[2] = new[2] + amount * (lum - new[2]);
        new.into()
    }

    /// Decrease saturation by 10%
    #[inline]
    pub fn desaturate(&self) -> Color {
        self.with_saturate(0.1)
    }

    /// Increase saturation by 10%
    #[inline]
    pub fn saturate(&self) -> Color {
        self.with_saturate(-0.1)
    }

    /// Returns color as hex format: #RRGGBBAA
    #[inline]
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    /// mid point between two colors
    pub fn mid(&self, other: &Color) -> Color {
        let midpoint = |lhs: u8, rhs: u8| {
            let half = (lhs as isize - rhs as isize).unsigned_abs() / 2;
            let min = lhs.min(rhs);
            if min as usize + half > 255 {
                255
            } else {
                min + half as u8
            }
        };

        Color {
            r: midpoint(self.r, other.r),
            g: midpoint(self.g, other.g),
            b: midpoint(self.b, other.b),
            a: midpoint(self.a, other.a),
        }
    }

    /// diff between two colors
    pub fn diff(&self, other: &Color) -> usize {
        (self.r as isize - other.r as isize).unsigned_abs()
            + (self.g as isize - other.g as isize).unsigned_abs()
            + (self.b as isize - other.b as isize).unsigned_abs()
            + (self.a as isize - other.a as isize).unsigned_abs()
    }
}

impl Tint for Color {
    #[inline]
    fn tint_add(&mut self, r_diff: isize, g_diff: isize, b_diff: isize, a_diff: isize) {
        let add = |current: u8, diff: isize| {
            let value = current as isize + diff;
            value.clamp(0, 255) as u8
        };
        self.r = add(self.r, r_diff);
        self.g = add(self.g, g_diff);
        self.b = add(self.b, b_diff);
        self.a = add(self.a, a_diff);
    }

    #[inline]
    fn tint_mul(&mut self, r_diff: f32, g_diff: f32, b_diff: f32, a_diff: f32) {
        let mul = |current: u8, diff: f32| {
            let value = current as f32 * diff;
            value.round().clamp(0., 255.) as u8
        };
        self.r = mul(self.r, r_diff);
        self.g = mul(self.g, g_diff);
        self.b = mul(self.b, b_diff);
        self.a = mul(self.a, a_diff);
    }
}

pub const WHITE: Color = Color::gray(255);
pub const OFF_WHITE: Color = Color::gray(250);
pub const BLACK: Color = Color::gray(0);
pub const OFF_BLACK: Color = Color::gray(5);
pub const DARKER_GRAY: Color = Color::gray(45);
pub const DARK_GRAY: Color = Color::gray(75);
pub const MID_GRAY: Color = Color::gray(110);
pub const LIGHT_GRAY: Color = Color::gray(180);
pub const LIGHTER_GRAY: Color = Color::gray(205);
pub const RED: Color = Color::new(255, 0, 0, 255);
pub const GREEN: Color = Color::new(0, 255, 0, 255);
pub const BLUE: Color = Color::new(0, 0, 255, 255);
pub const MAGENTA: Color = Color::new(255, 0, 255, 255);
pub const YELLOW: Color = Color::new(255, 255, 0, 255);
pub const ORANGE: Color = Color::new(255, 165, 0, 255);
pub const BROWN: Color = Color::new(139, 69, 19, 255);
pub const PURPLE: Color = Color::new(75, 0, 130, 255);
pub const CYAN: Color = Color::new(0, 255, 255, 255);
pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
/// Gameboy DMG-01 Foreground/Darkest
pub const GB_3: Color = Color::new(15, 56, 15, 255);
pub const GB_2: Color = Color::new(48, 98, 48, 255);
pub const GB_1: Color = Color::new(120, 145, 15, 255);
/// Gameboy DMG-01 Background/Lightest
pub const GB_0: Color = Color::new(155, 188, 15, 255);

#[cfg(test)]
mod test {
    use super::*;

    fn clone_and_add(initial: Color, r: isize, g: isize, b: isize, a: isize) -> Color {
        let mut color = initial;
        color.tint_add(r, g, b, a);
        color
    }

    fn clone_and_mul(initial: Color, r: f32, g: f32, b: f32, a: f32) -> Color {
        let mut color = initial;
        color.tint_mul(r, g, b, a);
        color
    }

    #[test]
    fn tint_add() {
        let initial = Color {
            r: 100,
            g: 150,
            b: 200,
            a: 255,
        };
        assert_eq!(
            clone_and_add(initial, 50, 50, 50, 0),
            Color::new(150, 200, 250, 255)
        );
        assert_eq!(
            clone_and_add(initial, 100, 100, 100, 0),
            Color::new(200, 250, 255, 255)
        );
        assert_eq!(
            clone_and_add(initial, -100, -100, -100, 0),
            Color::new(0, 50, 100, 255)
        );
        assert_eq!(
            clone_and_add(initial, 0, 0, 0, 0),
            Color::new(100, 150, 200, 255)
        );
        assert_eq!(
            clone_and_add(initial, 10, 0, 0, 0),
            Color::new(110, 150, 200, 255)
        );
        assert_eq!(
            clone_and_add(initial, 0, 10, 0, 0),
            Color::new(100, 160, 200, 255)
        );
        assert_eq!(
            clone_and_add(initial, 0, 0, 10, 0),
            Color::new(100, 150, 210, 255)
        );
        assert_eq!(
            clone_and_add(initial, 0, 0, 0, -10),
            Color::new(100, 150, 200, 245)
        );
        assert_eq!(
            clone_and_add(initial, 0, 0, 0, -500),
            Color::new(100, 150, 200, 0)
        );
    }

    #[test]
    fn tint_mul() {
        let initial = Color {
            r: 100,
            g: 150,
            b: 200,
            a: 255,
        };
        assert_eq!(
            clone_and_mul(initial, 1., 1., 1., 1.),
            Color::new(100, 150, 200, 255)
        );
        assert_eq!(
            clone_and_mul(initial, 0., 0., 0., 0.),
            Color::new(0, 0, 0, 0)
        );
        assert_eq!(
            clone_and_mul(initial, 2., 2., 2., 2.),
            Color::new(200, 255, 255, 255)
        );
        assert_eq!(
            clone_and_mul(initial, 0.5, 0.5, 0.5, 0.5),
            Color::new(50, 75, 100, 128)
        );
    }

    #[test]
    fn blend() {
        assert_eq!(
            Color::new(255, 255, 255, 255).blend(Color::new(0, 0, 0, 0)),
            Color::new(255, 255, 255, 255)
        );
        assert_eq!(
            Color::new(255, 255, 255, 255).blend(Color::new(255, 0, 0, 255)),
            Color::new(255, 0, 0, 255)
        );
        assert_eq!(
            Color::new(255, 255, 255, 255).blend(Color::new(255, 0, 0, 128)),
            Color::new(255, 127, 127, 255)
        );
        assert_eq!(
            Color::new(0, 0, 255, 128).blend(Color::new(255, 0, 0, 128)),
            Color::new(170, 0, 85, 192)
        );
    }

    #[test]
    fn from_hex() {
        assert_eq!(
            Color::from_hex("112233").unwrap(),
            Color {
                r: 17,
                g: 34,
                b: 51,
                a: 255,
            }
        );
        assert_eq!(
            Color::from_hex("#112233").unwrap(),
            Color {
                r: 17,
                g: 34,
                b: 51,
                a: 255,
            }
        );
        assert_eq!(
            Color::from_hex("#11223344").unwrap(),
            Color {
                r: 17,
                g: 34,
                b: 51,
                a: 68,
            }
        );
        assert!(Color::from_hex("#aafgha").is_err())
    }

    #[test]
    fn to_hex() {
        assert_eq!(WHITE.to_hex(), "#FFFFFFFF".to_string());
        assert_eq!(RED.to_hex(), "#FF0000FF".to_string());
    }

    #[test]
    fn brightness() {
        assert_eq!(WHITE.brightness(), 1.0);
        assert_eq!(BLACK.brightness(), 0.0);
        assert_eq!(Color::new(255, 0, 0, 255).brightness(), 0.2126);
        assert_eq!(Color::new(123, 0, 0, 255).brightness(), 0.102548234);
    }

    #[test]
    fn dark() {
        assert!(!WHITE.is_dark());
        assert!(BLACK.is_dark());
        assert!(!GREEN.is_dark());
        assert!(!CYAN.is_dark());
        assert!(RED.is_dark());
        assert!(DARK_GRAY.is_dark());
        assert!(!LIGHT_GRAY.is_dark());
    }

    #[test]
    fn _u32() {
        let num: u32 = RED.into();
        let color: Color = num.into();
        assert_eq!(RED, color);
    }
}
