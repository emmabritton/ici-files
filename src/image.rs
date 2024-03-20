#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::errors::IndexedImageError;
use crate::errors::IndexedImageError::*;
use crate::file::FileType::Image;
use crate::file::{verify_format, HEADER};
use crate::palette;
use crate::palette::FilePalette;
use crate::prelude::*;
use crate::scaling::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IndexedImage {
    width: u8,
    height: u8,
    palette: Vec<Color>,
    pixels: Vec<u8>,
    highest_palette_idx: u8,
}

impl IndexedImage {
    pub fn new(
        width: u8,
        height: u8,
        palette: Vec<Color>,
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

    pub fn blank(width: u8, height: u8, palette: Vec<Color>) -> Self {
        Self {
            width,
            height,
            palette,
            pixels: vec![0; width as usize * height as usize],
            highest_palette_idx: 0,
        }
    }
}

impl IndexedImage {
    /// Replace palette for image
    /// Will only return an error if the new palette has less colors than the image needs
    pub fn set_palette(&mut self, palette: &[Color]) -> Result<(), IndexedImageError> {
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
        palette: &[Color],
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
    pub fn set_palette_replace_color<C: Into<Color> + Copy>(
        &mut self,
        palette: &[Color],
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

    /// # Safety
    ///
    /// Out of bounds may occur
    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, pixel_idx: usize, color_idx: u8) {
        self.pixels[pixel_idx] = color_idx;
    }

    #[inline]
    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    #[inline]
    pub fn get_pixel(&self, pixel_idx: usize) -> Result<u8, IndexedImageError> {
        if pixel_idx >= self.pixels.len() {
            return Err(IndexOutOfRange(pixel_idx, self.pixels.len(), "pixels"));
        }
        Ok(self.pixels[pixel_idx])
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    #[inline]
    pub unsafe fn get_pixel_unchecked(&self, pixel_idx: usize) -> u8 {
        self.pixels[pixel_idx]
    }

    pub fn get_pixel_index(&self, x: u8, y: u8) -> Result<usize, IndexedImageError> {
        if x >= self.width {
            return Err(IndexOutOfRange(x as usize, self.width as usize, "width"));
        }
        if y >= self.height {
            return Err(IndexOutOfRange(y as usize, self.height as usize, "height"));
        }
        Ok(x as usize + y as usize * self.width as usize)
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    #[inline]
    pub unsafe fn get_pixel_index_unchecked(&self, x: u8, y: u8) -> usize {
        x as usize + y as usize * self.width as usize
    }

    #[inline]
    pub fn get_color(&self, idx: u8) -> Result<Color, IndexedImageError> {
        if idx >= self.palette.len() as u8 {
            return Err(IndexOutOfRange(idx as usize, self.palette.len(), "palette"));
        }
        Ok(self.palette[idx as usize])
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    #[inline]
    pub unsafe fn get_color_unchecked(&self, idx: u8) -> Color {
        self.palette[idx as usize]
    }

    #[inline]
    pub fn set_color(&mut self, idx: u8, color: Color) -> Result<(), IndexedImageError> {
        if idx >= self.palette.len() as u8 {
            return Err(IndexOutOfRange(idx as usize, self.palette.len(), "palette"));
        }
        self.palette[idx as usize] = color;
        Ok(())
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    #[inline]
    pub fn set_color_unchecked(&mut self, idx: u8, color: Color) {
        self.palette[idx as usize] = color;
    }

    #[inline]
    pub fn get_palette(&self) -> &[Color] {
        &self.palette
    }

    #[inline]
    pub fn min_palette_size_supported(&self) -> u8 {
        self.highest_palette_idx
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn rotate_cw(&self) -> IndexedImage {
        let mut output = IndexedImage::blank(self.height, self.width, self.palette.clone());
        for y in 0..self.height {
            for x in 0..self.width {
                let new_y = x;
                let new_x = output.width - y - 1;
                let new_i = output.get_pixel_index(new_x, new_y).unwrap();
                let i = output.get_pixel_index(x, y).unwrap();
                output.set_pixel(new_i, self.get_pixel(i).unwrap()).unwrap();
            }
        }
        output
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    pub unsafe fn rotate_cw_unchecked(&self) -> IndexedImage {
        let mut output = IndexedImage::blank(self.height, self.width, self.palette.clone());
        for y in 0..self.height {
            for x in 0..self.width {
                let new_y = x;
                let new_x = output.width - y - 1;
                let new_i = output.get_pixel_index_unchecked(new_x, new_y);
                let i = output.get_pixel_index_unchecked(x, y);
                output.set_pixel_unchecked(new_i, self.get_pixel_unchecked(i));
            }
        }
        output
    }

    pub fn rotate_ccw(&self) -> IndexedImage {
        let mut output = IndexedImage::blank(self.height, self.width, self.palette.clone());
        for y in 0..self.height {
            for x in 0..self.width {
                let new_y = output.height - x - 1;
                let new_x = y;
                let new_i = output.get_pixel_index(new_x, new_y).unwrap();
                let i = output.get_pixel_index(x, y).unwrap();
                output.set_pixel(new_i, self.get_pixel(i).unwrap()).unwrap();
            }
        }
        output
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    pub unsafe fn rotate_ccw_unchecked(&self) -> IndexedImage {
        let mut output = IndexedImage::blank(self.height, self.width, self.palette.clone());
        for y in 0..self.height {
            for x in 0..self.width {
                let new_y = output.height - x - 1;
                let new_x = y;
                let new_i = output.get_pixel_index_unchecked(new_x, new_y);
                let i = output.get_pixel_index_unchecked(x, y);
                output.set_pixel_unchecked(new_i, self.get_pixel_unchecked(i));
            }
        }
        output
    }

    pub fn flip_vertical(&self) -> Result<IndexedImage, IndexedImageError> {
        let mut output = IndexedImage::blank(self.width, self.height, self.palette.clone());
        for y in 0..self.height {
            let target_y = self.height - 1 - y;
            for x in 0..self.width {
                let target_i = output.get_pixel_index(x, target_y)?;
                let source_i = self.get_pixel_index(x, y)?;
                output.set_pixel(target_i, self.get_pixel(source_i)?)?;
            }
        }
        Ok(output)
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    pub unsafe fn flip_vertical_unchecked(&self) -> IndexedImage {
        let mut output = self.clone();
        let half_height = (output.height as f32 / 2.).floor() as u8;
        for y in 0..half_height {
            std::ptr::swap_nonoverlapping(
                &mut output.pixels[y as usize * output.width as usize],
                &mut output.pixels[(output.height - 1 - y) as usize * output.width as usize],
                output.width as usize,
            );
        }
        output
    }

    pub fn flip_horizontal(&self) -> Result<IndexedImage, IndexedImageError> {
        let mut output = IndexedImage::blank(self.width, self.height, self.palette.clone());
        let half_width = (self.width as f32 / 2.).floor() as u8;
        for y in 0..self.height {
            for x in 0..half_width {
                let target_right_i = output.get_pixel_index(self.width - x - 1, y)?;
                let source_left_i = self.get_pixel_index(x, y)?;
                let target_left_i = output.get_pixel_index(x, y)?;
                let source_right_i = self.get_pixel_index(self.width - 1 - x, y)?;
                let source_left = self.get_pixel(source_left_i)?;
                let source_right = self.get_pixel(source_right_i)?;

                output.set_pixel(target_left_i, source_right)?;
                output.set_pixel(target_right_i, source_left)?;
            }
        }
        Ok(output)
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    pub unsafe fn flip_horizontal_unchecked(&self) -> IndexedImage {
        let mut output = IndexedImage::blank(self.width, self.height, self.palette.clone());
        let half_width = (self.width as f32 / 2.).floor() as u8;
        for y in 0..self.height {
            for x in 0..half_width {
                let target_right_i = output.get_pixel_index_unchecked(self.width - 1 - x, y);
                let source_left_i = self.get_pixel_index_unchecked(x, y);
                let target_left_i = output.get_pixel_index_unchecked(x, y);
                let source_right_i = self.get_pixel_index_unchecked(self.width - 1 - x, y);
                let source_left = self.get_pixel_unchecked(source_left_i);
                let source_right = self.get_pixel_unchecked(source_right_i);

                output.set_pixel_unchecked(target_left_i, source_right);
                output.set_pixel_unchecked(target_right_i, source_left);
            }
        }
        output
    }

    pub fn scale(&self, algo: Scaling) -> Result<IndexedImage, IndexedImageError> {
        match algo {
            Scaling::NearestNeighbour { x_scale, y_scale } => {
                scale_nearest_neighbor(self, usize::from(x_scale), usize::from(y_scale))
            }
            Scaling::Epx2x => scale_epx(self),
            Scaling::Epx4x => scale_epx(&scale_epx(self)?),
        }
    }

    /// # Safety
    ///
    /// Out of bounds may occur
    pub unsafe fn scale_unchecked(&self, algo: Scaling) -> IndexedImage {
        match algo {
            Scaling::NearestNeighbour { x_scale, y_scale } => {
                scale_nearest_neighbor_unchecked(self, usize::from(x_scale), usize::from(y_scale))
            }
            Scaling::Epx2x => scale_epx_unchecked(self),
            Scaling::Epx4x => scale_epx_unchecked(&scale_epx_unchecked(self)),
        }
    }

    pub fn tint_palette_add(&self, color_diff: &[(isize, isize, isize, isize)]) -> IndexedImage {
        let mut output = self.clone();

        for (i, color) in output.palette.iter_mut().enumerate() {
            let diff = color_diff[i];
            color.tint_add(diff.0, diff.1, diff.2, diff.3)
        }

        output
    }

    pub fn tint_palette_mut(&self, color_diff: &[(f32, f32, f32, f32)]) -> IndexedImage {
        let mut output = self.clone();

        for (i, color) in output.palette.iter_mut().enumerate() {
            let diff = color_diff[i];
            color.tint_mul(diff.0, diff.1, diff.2, diff.3)
        }

        output
    }

    pub fn tint_add(&self, color_diff: &(isize, isize, isize, isize)) -> IndexedImage {
        let mut output = self.clone();

        for color in output.palette.iter_mut() {
            color.tint_add(color_diff.0, color_diff.1, color_diff.2, color_diff.3)
        }

        output
    }

    pub fn tint_mul(&self, color_diff: &(f32, f32, f32, f32)) -> IndexedImage {
        let mut output = self.clone();

        for color in output.palette.iter_mut() {
            color.tint_mul(color_diff.0, color_diff.1, color_diff.2, color_diff.3)
        }

        output
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
            None => vec![TRANSPARENT; highest + 1],
            Some(colors) => colors,
        };

        IndexedImage::new(width, height, colors, pixels.to_vec()).map(|image| (image, pal_type))
    }
}

#[cfg(test)]
mod test {
    use crate::palette::FilePalette::*;

    use super::*;

    #[test]
    fn write_and_read_no_data() {
        let width = 2;
        let height = 2;
        let input = IndexedImage::new(
            2,
            2,
            vec![
                TRANSPARENT,
                Color::new(50, 51, 52, 53),
                Color::new(60, 61, 62, 63),
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
                2,
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[TRANSPARENT, TRANSPARENT, TRANSPARENT])
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
                TRANSPARENT,
                Color::new(50, 51, 52, 53),
                Color::new(60, 61, 62, 63),
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
                2,
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[TRANSPARENT, TRANSPARENT, TRANSPARENT])
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
                TRANSPARENT,
                Color::new(50, 51, 52, 53),
                Color::new(60, 61, 62, 63),
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
                2,
            ]
        );
        let (output, pal) = IndexedImage::from_file_contents(&bytes).unwrap();
        let mut cloned = input.clone();
        cloned
            .set_palette(&[TRANSPARENT, TRANSPARENT, TRANSPARENT])
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
                TRANSPARENT,
                Color::new(50, 51, 52, 53),
                Color::new(60, 61, 62, 63),
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
                2,
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
            vec![Color::new(255, 255, 255, 255), Color::new(0, 0, 0, 255)],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified
            .set_palette(&[Color::new(255, 0, 0, 255), Color::new(0, 255, 0, 255)])
            .unwrap();
        assert_eq!(image.highest_palette_idx, modified.highest_palette_idx);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(
            modified.palette,
            vec![Color::new(255, 0, 0, 255), Color::new(0, 255, 0, 255)]
        );
    }

    #[test]
    fn set_palette_id() {
        let image = IndexedImage::new(
            2,
            4,
            vec![
                Color::new(1, 1, 1, 1),
                Color::new(2, 2, 2, 2),
                Color::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified
            .set_palette_replace_id(&[Color::new(5, 5, 5, 5)], 0)
            .unwrap();
        assert_eq!(modified.highest_palette_idx, 0);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(modified.palette, vec![Color::new(5, 5, 5, 5)]);
    }

    #[test]
    fn set_palette_color() {
        let image = IndexedImage::new(
            2,
            4,
            vec![
                Color::new(1, 1, 1, 1),
                Color::new(2, 2, 2, 2),
                Color::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0],
        )
        .unwrap();
        let mut modified = image.clone();
        modified.set_palette_replace_color(&[Color::new(5, 5, 5, 5)], Color::new(5, 5, 5, 5));
        assert_eq!(modified.highest_palette_idx, 2);
        assert_eq!(image.height, modified.height);
        assert_eq!(image.width, modified.width);
        assert_eq!(image.pixels, image.pixels);
        assert_eq!(
            modified.palette,
            vec![
                Color::new(5, 5, 5, 5),
                Color::new(5, 5, 5, 5),
                Color::new(5, 5, 5, 5),
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
                TRANSPARENT,
                Color::new(50, 51, 52, 53),
                Color::new(60, 61, 62, 63),
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
                Color::new(1, 1, 1, 1),
                Color::new(2, 2, 2, 2),
                Color::new(3, 3, 3, 3),
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
