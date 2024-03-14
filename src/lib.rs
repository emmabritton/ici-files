pub mod animated;
pub mod color;
pub mod errors;
pub mod file;
pub mod image;
pub mod jasc_palette;
pub mod palette;
pub mod scaling;
pub mod wrapper;

pub mod prelude {
    pub use crate::animated::*;
    pub use crate::color::*;
    pub use crate::errors::*;
    pub use crate::image::*;
    pub use crate::jasc_palette::*;
    pub use crate::palette::FilePalette;
    pub use crate::scaling::*;
    pub use crate::wrapper::*;
    pub use crate::*;
}

pub trait Tint {
    /// Add to the RGBA channels by the amounts specified
    ///
    /// Channels are clamped to 0..=255
    fn tint_add(&mut self, r_diff: isize, g_diff: isize, b_diff: isize, a_diff: isize);
    /// Multiply the RGBA channels by the amounts specified
    ///
    /// Channels are clamped to 0..=255
    fn tint_mul(&mut self, r_diff: f32, g_diff: f32, b_diff: f32, a_diff: f32);
}
