use crate::errors::IndexedImageError;
use crate::errors::IndexedImageError::*;
use crate::file::FileType::*;

//last is file version
pub(crate) const HEADER: [u8; 4] = [b'I', b'C', b'I', 1];

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FileType {
    Image,
    Animated,
}

impl FileType {
    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            Image => 1,
            Animated => 2,
        }
    }

    pub(crate) fn from_byte(byte: u8) -> Option<FileType> {
        match byte {
            1 => Some(Image),
            2 => Some(Animated),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Image => "Image",
            Animated => "Animated Image",
        }
    }

    pub fn ext(&self) -> &'static str {
        match self {
            Image => "ici",
            Animated => "ica",
        }
    }
}

pub(super) fn verify_format(bytes: &[u8]) -> Result<FileType, IndexedImageError> {
    if bytes.len() < 10 {
        return Err(NotIciFile);
    }
    if bytes[0..HEADER.len()] != HEADER {
        return Err(NotIciFile);
    }
    let format = bytes[HEADER.len()];
    match FileType::from_byte(format) {
        None => Err(UnknownIciVersion(format)),
        Some(file_type) => Ok(file_type),
    }
}
