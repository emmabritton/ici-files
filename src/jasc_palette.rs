use crate::jasc_palette::PaletteError::*;
use crate::jasc_palette::ParseIssue::*;
use crate::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PaletteError {
    InvalidFileType,
    UnsupportedVersion,
    IncorrectNumberOfColors,
    ParseError(ParseIssue),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParseIssue {
    FileDesc,
    Version,
    ColorCount,
    ColorSplitting(usize),
    ColorNumbers(usize),
}

impl Display for PaletteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidFileType => write!(f, "Invalid file type"),
            UnsupportedVersion => write!(f, "Palette file version is not supported"),
            IncorrectNumberOfColors => write!(f, "Palette file has the wrong number of colors"),
            ParseError(reason) => match reason {
                FileDesc => write!(f, "Error parsing the file type descriptor"),
                Version => write!(f, "Error parsing the version"),
                ColorCount => write!(f, "Error parsing the color count"),
                ColorSplitting(num) => write!(f, "Error splitting color {num}"),
                ColorNumbers(num) => write!(f, "Error parsing color {num}"),
            },
        }
    }
}

impl Error for PaletteError {}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JascPalette {
    pub colors: Vec<IciColor>,
}

impl JascPalette {
    pub const fn new(colors: Vec<IciColor>) -> Self {
        Self { colors }
    }

    pub fn from(colors: &[IciColor]) -> Self {
        Self {
            colors: colors.to_vec(),
        }
    }
}

const FILE_HEADER: &str = "JASC-PAL";
const FILE_VER: &str = "0100";

impl JascPalette {
    pub fn to_file_contents(&self) -> String {
        let mut output = String::new();
        output.push_str(FILE_HEADER);
        output.push('\n');
        output.push_str(FILE_VER);
        output.push('\n');
        output.push_str(&self.colors.len().to_string());
        output.push('\n');
        for color in &self.colors {
            output.push_str(&color.r.to_string());
            output.push(' ');
            output.push_str(&color.g.to_string());
            output.push(' ');
            output.push_str(&color.b.to_string());
            if color.a != 255 {
                output.push(' ');
                output.push_str(&color.a.to_string());
            }
            output.push('\n');
        }

        output
    }

    pub fn from_file_contents(text: &str) -> Result<JascPalette, PaletteError> {
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            if line != FILE_HEADER {
                return Err(InvalidFileType);
            }
        } else {
            return Err(ParseError(FileDesc));
        }
        if let Some(line) = lines.next() {
            if line != FILE_VER {
                return Err(UnsupportedVersion);
            }
        } else {
            return Err(ParseError(Version));
        }
        let count = if let Some(line) = lines.next() {
            match u8::from_str(line) {
                Ok(num) => num,
                Err(_) => return Err(ParseError(ColorCount)),
            }
        } else {
            return Err(ParseError(ColorCount));
        };
        let colors: Vec<&str> = lines.collect();
        if colors.len() as u8 != count {
            return Err(IncorrectNumberOfColors);
        }
        let mut output = vec![];
        for (i, color) in colors.iter().enumerate() {
            let values: Vec<&str> = color.split_whitespace().collect();
            if values.len() != 3 && values.len() != 4 {
                return Err(ParseError(ColorSplitting(i)));
            }
            let r = u8::from_str(values[0]).map_err(|_| ParseError(ColorNumbers(i)))?;
            let g = u8::from_str(values[1]).map_err(|_| ParseError(ColorNumbers(i)))?;
            let b = u8::from_str(values[2]).map_err(|_| ParseError(ColorNumbers(i)))?;
            let mut a = 255;
            if values.len() == 4 {
                a = u8::from_str(values[3]).map_err(|_| ParseError(ColorNumbers(i)))?;
            }
            output.push(IciColor { r, g, b, a })
        }
        Ok(JascPalette::new(output))
    }
}
