pub mod animated;
pub mod errors;
pub mod file;
pub mod image;
pub mod palette;

pub mod prelude {
    pub use crate::animated::*;
    pub use crate::errors::*;
    pub use crate::image::*;
    pub use crate::palette::FilePalette;
    pub use crate::*;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct IciColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl IciColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn transparent() -> IciColor {
        IciColor::new(0, 0, 0, 0)
    }
}

//
// #[derive(Debug, Clone)]
// pub struct ImageAtlas {
//     columns: NonZeroU8,
//     rows: NonZeroU8,
//     width: NonZeroU8,
//     height: NonZeroU8,
//     palette: Vec<IciColor>,
//     tiles: Vec<Vec<u8>>,
//     named: Vec<NamedTile>
// }
//
// #[derive(Debug, Clone)]
// pub struct NamedTile {
//     index: u8,
//     name: String,
//     width: NonZeroU8,
//     height: NonZeroU8
// }
