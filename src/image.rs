use crate::errors::IndexedImageError;
use crate::errors::IndexedImageError::*;
use crate::file::FileType::Image;
use crate::file::{verify_format, HEADER};
use crate::palette::FilePalette;
use crate::{palette, IciColor};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedImage {
    width: u8,
    height: u8,
    palette: Vec<IciColor>,
    pixels: Vec<u8>,
    highest_palette_idx: u8,
}

impl IndexedImage {
    pub fn new(
        width: u8,
        height: u8,
        palette: Vec<IciColor>,
        pixels: Vec<u8>,
    ) -> Result<Self, IndexedImageError> {
        if width == 0 {
            return Err(WidthIsZero);
        }
        if height == 0 {
            return Err(HeightIsZero);
        }
        if palette.is_empty() {
            return Err(PaletteIsEmpty);
        }
        if pixels.len() != (width as usize * height as usize) {
            return Err(MissingData(pixels.len(), width as usize * height as usize));
        }
        let highest_palette_idx = *pixels
            .iter()
            .max()
            .expect("Unable to get highest color index");
        Ok(Self {
            width,
            height,
            palette,
            pixels,
            highest_palette_idx,
        })
    }
}

impl IndexedImage {
    /// Replace palette for image
    /// Will only return an error if the new palette has less colors than the image needs
    pub fn set_palette(&mut self, palette: &[IciColor]) -> Result<(), IndexedImageError> {
        assert!(!palette.is_empty());
        if palette.len() < self.highest_palette_idx as usize {
            return Err(PaletteTooFewColors(self.highest_palette_idx));
        }
        self.palette = palette.to_vec();
        Ok(())
    }

    /// Replace palette for image, any pixels outside the new palette will be replaced with `id`
    /// Will only return an error if id is outside the new palette
    pub fn set_palette_replace_id(
        &mut self,
        palette: &[IciColor],
        id: u8,
    ) -> Result<(), IndexedImageError> {
        let new_palette_len = palette.len() as u8;
        assert!(!palette.is_empty());
        if new_palette_len <= id {
            return Err(IdOutsideOfNewPalette);
        }
        self.palette = palette.to_vec();
        for i in self.pixels.iter_mut() {
            if *i >= new_palette_len {
                *i = id;
            }
        }
        self.highest_palette_idx = *self
            .pixels
            .iter()
            .max()
            .expect("Unable to get highest color index");
        Ok(())
    }

    /// Replace palette for image, any color indexes outside the palette will be expanded with `color`
    pub fn set_palette_replace_color<C: Into<IciColor> + Copy>(
        &mut self,
        palette: &[IciColor],
        color: C,
    ) {
        assert!(!palette.is_empty());
        let mut tmp_pal = palette.to_vec();
        while tmp_pal.len() <= self.highest_palette_idx as usize {
            tmp_pal.push(color.into());
        }
        self.palette = tmp_pal;
    }

    #[inline]
    pub fn size(&self) -> (u8, u8) {
        (self.width, self.height)
    }

    #[inline]
    pub fn set_pixel(&mut self, pixel_idx: usize, color_idx: u8) -> Result<(), IndexedImageError> {
        if pixel_idx >= self.pixels.len() {
            return Err(IndexOutOfRange(pixel_idx, self.pixels.len(), "pixels"));
        }
        if color_idx >= self.palette.len() as u8 {
            return Err(IndexOutOfRange(
                color_idx as usize,
                self.palette.len(),
                "palette",
            ));
        }
        self.pixels[pixel_idx] = color_idx;
        Ok(())
    }

    #[inline]
    pub fn get_pixel(&self, pixel_idx: usize) -> Result<u8, IndexedImageError> {
        if pixel_idx >= self.pixels.len() {
            return Err(IndexOutOfRange(pixel_idx, self.pixels.len(), "pixels"));
        }
        Ok(self.pixels[pixel_idx])
    }

    #[inline]
    pub fn get_pixel_index(&self, x: u8, y: u8) -> Result<usize, IndexedImageError> {
        if x >= self.width {
            return Err(IndexOutOfRange(x as usize, self.width as usize, "width"));
        }
        if y >= self.height {
            return Err(IndexOutOfRange(y as usize, self.height as usize, "height"));
        }
        Ok(x as usize + y as usize * self.width as usize)
    }

    #[inline]
    pub fn get_color(&self, idx: u8) -> Result<IciColor, IndexedImageError> {
        if idx >= self.palette.len() as u8 {
            return Err(IndexOutOfRange(idx as usize, self.palette.len(), "palette"));
        }
        Ok(self.palette[idx as usize])
    }

    #[inline]
    pub fn set_color(&mut self, idx: u8, color: IciColor) -> Result<(), IndexedImageError> {
        if idx >= self.palette.len() as u8 {
            return Err(IndexOutOfRange(idx as usize, self.palette.len(), "palette"));
        }
        self.palette[idx as usize] = color;
        Ok(())
    }

    #[inline]
    pub fn get_palette(&self) -> &[IciColor] {
        &self.palette
    }

    #[inline]
    pub fn min_palette_size_supported(&self) -> u8 {
        self.highest_palette_idx
    }
}

impl IndexedImage {
    /// Errors will only be returned if you [FilePalette::Name] and the len is invalid
    pub fn to_file_contents(&self, palette: &FilePalette) -> Result<Vec<u8>, IndexedImageError> {
        let mut output = vec![];
        output.extend_from_slice(&HEADER);
        output.push(Image.to_byte());

        palette::write(palette, self.get_palette(), &mut output)?;
        output.push(self.width);
        output.push(self.height);
        output.extend_from_slice(&self.pixels);

        Ok(output)
    }

    /// Create an [IndexedImage], image palette will be filled with transparency unless file contains colors
    /// use `image.set_palette*` to replace the palette
    pub fn from_file_contents(
        bytes: &[u8],
    ) -> Result<(IndexedImage, FilePalette), IndexedImageError> {
        let file_type = verify_format(bytes)?;
        if file_type != Image {
            return Err(InvalidFileFormat(
                0,
                format!("Expected Image file but found {}", file_type.name()),
            ));
        }
        let idx = HEADER.len() + 1;
        let (skip, pal_type, colors) = palette::read(idx, bytes)?;

        let start = idx + skip;
        if bytes.len() < start + 3 {
            return Err(InvalidFileFormat(
                start,
                "Incomplete pixels data".to_string(),
            ));
        }
        let width = bytes[start];
        let height = bytes[start + 1];
        let pixels_len = width as usize * height as usize;
        if bytes.len() < start + 2 + pixels_len {
            return Err(InvalidFileFormat(
                start + 2,
                format!(
                    "Incomplete pixels data, found {} but expected {}",
                    pixels_len,
                    width * height
                ),
            ));
        }
        let pixels = &bytes[start + 2..start + 2 + pixels_len];

        let highest = *pixels.iter().max().expect("Invalid pixels data") as usize;
        let colors = match colors {
            None => vec![IciColor::transparent(); highest + 1],
            Some(colors) => colors,
        };

        IndexedImage::new(width, height, colors, pixels.to_vec()).map(|image| (image, pal_type))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::palette::FilePalette::*;

    #[test]
    fn write_and_read_no_data() {
        let width = 2;
        let height = 2;
        let input = IndexedImage::new(
            2,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2],
        )
        .unwrap();
        let bytes = input.to_file_contents(&NoData).unwrap();
        assert_eq!(
            bytes,
            vec![
                HEADER[0],
                HEADER[1],
                HEADER[2],
                HEADER[3],
                Image.to_byte(),
                NoData.to_byte(),
                width,
                height,
                0,
                0,
                1,
                2
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[
                IciColor::transparent(),
                IciColor::transparent(),
                IciColor::transparent(),
            ])
            .unwrap();
        assert_eq!(cloned, output);
        assert_eq!(pal, NoData);
    }

    #[test]
    fn write_and_read_id() {
        let width = 2;
        let height = 2;
        let input = IndexedImage::new(
            2,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2],
        )
        .unwrap();
        let bytes = input.to_file_contents(&ID(15)).unwrap();
        assert_eq!(
            bytes,
            vec![
                HEADER[0],
                HEADER[1],
                HEADER[2],
                HEADER[3],
                Image.to_byte(),
                ID(0).to_byte(),
                0,
                15,
                width,
                height,
                0,
                0,
                1,
                2
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[
                IciColor::transparent(),
                IciColor::transparent(),
                IciColor::transparent(),
            ])
            .unwrap();
        assert_eq!(cloned, output);
        assert_eq!(pal, ID(15));
    }

    #[test]
    fn write_and_read_name() {
        let width = 2;
        let height = 2;
        let input = IndexedImage::new(
            2,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2],
        )
        .unwrap();
        let bytes = input.to_file_contents(&Name("Test".to_string())).unwrap();
        assert_eq!(
            bytes,
            vec![
                HEADER[0],
                HEADER[1],
                HEADER[2],
                HEADER[3],
                Image.to_byte(),
                Name(String::new()).to_byte(),
                4,
                b'T',
                b'e',
                b's',
                b't',
                width,
                height,
                0,
                0,
                1,
                2
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[
                IciColor::transparent(),
                IciColor::transparent(),
                IciColor::transparent(),
            ])
            .unwrap();
        assert_eq!(cloned, output);
        assert_eq!(pal, Name("Test".to_string()));
    }

    #[test]
    fn write_and_read_colors() {
        let width = 2;
        let height = 2;
        let input = IndexedImage::new(
            2,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2],
        )
        .unwrap();
        let bytes = input.to_file_contents(&Colors).unwrap();
        assert_eq!(
            bytes,
            vec![
                HEADER[0],
                HEADER[1],
                HEADER[2],
                HEADER[3],
                Image.to_byte(),
                Colors.to_byte(),
                3,
                0,
                0,
                0,
                0,
                50,
                51,
                52,
                53,
                60,
                61,
                62,
                63,
                width,
                height,
                0,
                0,
                1,
                2
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        assert_eq!(input, output);
        assert_eq!(pal, Colors);
    }

    #[test]
    fn set_palette() {
        let image = IndexedImage::new(
            3,
            3,
            vec![
                IciColor::new(255, 255, 255, 255),
                IciColor::new(0, 0, 0, 255),
            ],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified
            .set_palette(&[IciColor::new(255, 0, 0, 255), IciColor::new(0, 255, 0, 255)])
            .unwrap();
        assert_eq!(image.highest_palette_idx, modified.highest_palette_idx);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(
            modified.palette,
            vec![IciColor::new(255, 0, 0, 255), IciColor::new(0, 255, 0, 255)]
        );
    }

    #[test]
    fn set_palette_id() {
        let image = IndexedImage::new(
            2,
            4,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified
            .set_palette_replace_id(&[IciColor::new(5, 5, 5, 5)], 0)
            .unwrap();
        assert_eq!(modified.highest_palette_idx, 0);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(modified.palette, vec![IciColor::new(5, 5, 5, 5)]);
    }

    #[test]
    fn set_palette_color() {
        let image = IndexedImage::new(
            2,
            4,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified.set_palette_replace_color(&[IciColor::new(5, 5, 5, 5)], IciColor::new(5, 5, 5, 5));
        assert_eq!(modified.highest_palette_idx, 2);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(
            modified.palette,
            vec![
                IciColor::new(5, 5, 5, 5),
                IciColor::new(5, 5, 5, 5),
                IciColor::new(5, 5, 5, 5)
            ]
        );
    }

    #[test]
    fn test_ignores_extra_data() {
        let data = vec![
            HEADER[0],
            HEADER[1],
            HEADER[2],
            HEADER[3],
            Image.to_byte(),
            Colors.to_byte(),
            3,
            0,
            0,
            0,
            0,
            50,
            51,
            52,
            53,
            60,
            61,
            62,
            63,
            2,
            2,
            0,
            0,
            1,
            2,
            100,
            100,
            100,
        ];
        let image = IndexedImage::new(
            2,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2],
        )
        .unwrap();
        let input = IndexedImage::from_file_contents(&data).unwrap();
        assert_eq!(image, input.0);
    }

    #[test]
    fn pixel_accessors() {
        let mut image = IndexedImage::new(
            2,
            4,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0],
        )
        .unwrap();
        let idx = image.get_pixel_index(1, 2).unwrap();
        assert!(image.get_pixel_index(4, 4).is_err());
        assert_eq!(image.get_pixel(idx).unwrap(), 1);
        assert!(image.set_pixel(idx, 2).is_ok());
        assert_eq!(image.get_pixel(idx).unwrap(), 2);
    }
}
