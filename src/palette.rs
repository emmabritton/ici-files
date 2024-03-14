use crate::errors::IndexedImageError::*;
use crate::palette::FilePalette::*;
use crate::prelude::*;
use std::collections::HashSet;

pub(crate) const PAL_NO_DATA: u8 = 0;
pub(crate) const PAL_ID: u8 = 1;
pub(crate) const PAL_NAME: u8 = 2;
pub(crate) const PAL_COLORS: u8 = 3;

/// How palette data is stored in an ICI file
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FilePalette {
    /// Include no palette information
    NoData,
    /// Include palette id (reader will need to know what the id refers to)
    ID(u16),
    /// Include palette name (reader will need to know what the name refers to) 1..=255 chars
    Name(String),
    /// Include palette colors
    Colors,
}

fn distinct_count(colors: &[Color]) -> usize {
    colors.iter().collect::<HashSet<_>>().len()
}

/// merges similar colors until there are < `max` unique colors
/// the result will contain duplicates so the index is preserved
pub fn simplify_palette_to_fit(colors: &[Color], max: usize) -> Vec<Color> {
    let mut output = colors.to_vec();
    let mut threshold = 2;
    while distinct_count(&output) >= max {
        output = simplify_palette(&output, threshold);
        threshold += 10;
    }
    output
}

/// merges similar colors that fall with threshold of each other
/// will remove gradients if threshold is too high or gradient too smooth
///
/// recommend starting threshold at 2, max is 1020
/// the result will contain duplicates so the index is preserved
pub fn simplify_palette(colors: &[Color], threshold: usize) -> Vec<Color> {
    let mut output = colors.to_vec();
    let mut idx = 0;

    loop {
        let color = &output[idx];
        let mut to_merge = None;
        for (i, cmp_color) in output.iter().enumerate() {
            let diff = color.diff(cmp_color);
            if idx != i && diff > 0 && diff < threshold {
                to_merge = Some(i);
                break;
            }
        }
        if let Some(i) = to_merge {
            let c = output[idx].mid(&output[i]);
            output[idx] = c;
            output[i] = c;
        } else {
            idx += 1;
            if idx >= output.len() {
                break;
            }
        }
    }

    output
}

impl FilePalette {
    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            NoData => PAL_NO_DATA,
            ID(_) => PAL_ID,
            Name(_) => PAL_NAME,
            Colors => PAL_COLORS,
        }
    }
}

pub(crate) fn write(
    palette: &FilePalette,
    colors: &[Color],
    output: &mut Vec<u8>,
) -> Result<(), IndexedImageError> {
    output.push(palette.to_byte());
    match palette {
        NoData => {}
        ID(id) => output.extend_from_slice(&id.to_be_bytes()),
        Name(name) => {
            let len = name.len();
            if len < 1 {
                return Err(PaletteNameTooShort);
            }
            if len > 255 {
                return Err(PaletteNameTooLong);
            }
            output.push(len as u8);
            output.extend_from_slice(name.as_bytes())
        }
        Colors => {
            output.push(colors.len() as u8);
            for color in colors {
                output.push(color.r);
                output.push(color.g);
                output.push(color.b);
                output.push(color.a);
            }
        }
    }

    Ok(())
}

pub(crate) fn read(
    mut start_idx: usize,
    bytes: &[u8],
) -> Result<(usize, FilePalette, Option<Vec<Color>>), IndexedImageError> {
    if bytes.len() <= start_idx {
        return Err(InvalidFileFormat(
            start_idx,
            "No data after header, expected palette format".to_string(),
        ));
    }
    let pal_type = bytes[start_idx];
    start_idx += 1;
    match pal_type {
        PAL_NO_DATA => Ok((1, NoData, None)),
        PAL_ID => {
            if bytes.len() < start_idx + 1 {
                return Err(InvalidFileFormat(
                    start_idx,
                    "No data after palette format, expected ID".to_string(),
                ));
            }
            let bytes = &bytes[start_idx..=start_idx + 1];
            let id = u16::from_be_bytes([bytes[0], bytes[1]]);
            Ok((3, ID(id), None))
        }
        PAL_NAME => {
            if bytes.len() < start_idx {
                return Err(InvalidFileFormat(
                    start_idx,
                    "No data after palette format, expected palette name length".to_string(),
                ));
            }
            let len = bytes[start_idx];
            start_idx += 1;
            let end = len as usize;
            if bytes.len() < start_idx + end {
                return Err(InvalidFileFormat(
                    start_idx,
                    "Incomplete data after palette name length, expected palette name".to_string(),
                ));
            }
            let name = String::from_utf8(bytes[start_idx..start_idx + end].to_vec())
                .map_err(PaletteNameNotUtf8)?;
            Ok((end + 2, Name(name), None))
        }
        PAL_COLORS => {
            if bytes.len() < start_idx {
                return Err(InvalidFileFormat(
                    start_idx,
                    "No data after palette format, expected color count".to_string(),
                ));
            }
            let count = bytes[start_idx];
            start_idx += 1;
            let end = count as usize * 4;
            if bytes.len() < start_idx + end {
                return Err(InvalidFileFormat(
                    start_idx,
                    format!("Incomplete data after palette color count, expected {count} colors"),
                ));
            }
            let mut colors = vec![];
            let color_bytes: Vec<&u8> = bytes.iter().skip(start_idx).take(end).collect();
            for color in color_bytes.chunks_exact(4) {
                colors.push(Color::new(*color[0], *color[1], *color[2], *color[3]));
            }
            Ok((end + 2, Colors, Some(colors)))
        }
        _ => Err(InvalidFileFormat(
            start_idx,
            format!("Unsupport palette type {pal_type}"),
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write_no_data() {
        let mut output = vec![];
        write(&NoData, &[], &mut output).unwrap();
        assert_eq!(output, vec![PAL_NO_DATA]);

        let mut output = vec![];
        write(&NoData, &[Color::new(255, 45, 231, 2)], &mut output).unwrap();
        assert_eq!(output, vec![PAL_NO_DATA]);
    }

    #[test]
    fn write_id() {
        let mut output = vec![];
        write(&ID(5), &[], &mut output).unwrap();
        assert_eq!(output, vec![PAL_ID, 0, 5]);

        let mut output = vec![];
        write(&ID(256), &[Color::new(255, 45, 231, 2)], &mut output).unwrap();
        assert_eq!(output, vec![PAL_ID, 1, 0]);
    }

    #[test]
    fn write_name() {
        let mut output = vec![];
        write(&Name("test".to_string()), &[], &mut output).unwrap();
        assert_eq!(output, vec![PAL_NAME, 4, b't', b'e', b's', b't']);

        let mut output = vec![];
        write(
            &Name("😺".to_string()),
            &[Color::new(255, 45, 231, 2)],
            &mut output,
        )
        .unwrap();
        assert_eq!(output, vec![PAL_NAME, 4, 240, 159, 152, 186]);
    }

    #[test]
    fn write_colors() {
        let mut output = vec![];
        write(&Colors, &[Color::new(100, 101, 102, 103)], &mut output).unwrap();
        assert_eq!(output, vec![PAL_COLORS, 1, 100, 101, 102, 103]);

        let mut output = vec![];
        write(
            &Colors,
            &[Color::new(100, 101, 102, 103), Color::new(0, 0, 0, 255)],
            &mut output,
        )
        .unwrap();
        assert_eq!(
            output,
            vec![PAL_COLORS, 2, 100, 101, 102, 103, 0, 0, 0, 255]
        );
    }

    #[test]
    fn read_no_data() {
        let (skip, pal_type, colors) = read(0, &[PAL_NO_DATA]).unwrap();
        assert_eq!(skip, 1);
        assert_eq!(pal_type, NoData);
        assert_eq!(colors, None);
    }

    #[test]
    fn read_id() {
        let (skip, pal_type, colors) = read(0, &[PAL_ID, 0, 5]).unwrap();
        assert_eq!(skip, 3);
        assert_eq!(pal_type, ID(5));
        assert_eq!(colors, None);
    }

    #[test]
    fn read_name() {
        let (skip, pal_type, colors) = read(0, &[PAL_NAME, 4, 240, 159, 152, 186]).unwrap();
        assert_eq!(skip, 6);
        assert_eq!(pal_type, Name("😺".to_string()));
        assert_eq!(colors, None);
    }

    #[test]
    fn read_colors() {
        let (skip, pal_type, colors) =
            read(0, &[PAL_COLORS, 2, 100, 101, 102, 103, 0, 0, 0, 255]).unwrap();
        assert_eq!(skip, 10);
        assert_eq!(pal_type, Colors);
        assert_eq!(
            colors,
            Some(vec![
                Color::new(100, 101, 102, 103),
                Color::new(0, 0, 0, 255)
            ])
        );
    }

    #[test]
    fn write_data_before() {
        let mut output = vec![1, 1, 1, 1];
        write(&ID(5), &[], &mut output).unwrap();
        assert_eq!(output, vec![1, 1, 1, 1, PAL_ID, 0, 5]);
    }

    #[test]
    fn read_data_either_side() {
        let bytes = [
            1, 1, 1, 1, PAL_COLORS, 2, 100, 101, 102, 103, 0, 0, 0, 255, 2, 2, 2, 2,
        ];
        let start = 4;
        let (skip, pal_type, colors) = read(start, &bytes).unwrap();
        assert_eq!(skip, 10);
        assert_eq!(pal_type, Colors);
        assert_eq!(
            colors,
            Some(vec![
                Color::new(100, 101, 102, 103),
                Color::new(0, 0, 0, 255)
            ])
        );
        assert_eq!(bytes[start + skip..], [2, 2, 2, 2]);
    }
}
