use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexedImageError {
    #[error("Invalid file header")]
    NotIciFile,
    #[error("Unsupported ICI version {0}")]
    UnknownIciVersion(u8),
    #[error("Palette name must have at least 1 character")]
    PaletteNameTooShort,
    #[error("Palette name has more than 255 character")]
    PaletteNameTooLong,
    #[error("Palette has more than 255 colors")]
    PaletteTooManyColors,
    #[error("Image requires a palette with at least {0} colors")]
    PaletteTooFewColors(u8),
    #[error("Invalid file format/contents at {0}: {1}")]
    InvalidFileFormat(usize, String),
    #[error("Palette name was not valid UTF-8")]
    PaletteNameNotUtf8(#[from] FromUtf8Error),
    #[error("ID was greater than palette size")]
    IdOutsideOfNewPalette,
    #[error("Index {0} was outside of {2} (len {1})")]
    IndexOutOfRange(usize, usize, &'static str),
    #[error("Image width is 0")]
    WidthIsZero,
    #[error("Image height is 0")]
    HeightIsZero,
    #[error("Missing pixels data, count: {0} expected: {1}")]
    MissingData(usize, usize),
    #[error("Palette is empty")]
    PaletteIsEmpty,
    #[error("Per frame timing is negative: {0}")]
    NegativePerFrame(f64),
}
