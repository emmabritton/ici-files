use crate::animated::PlayType::*;
use crate::errors::IndexedImageError;
use crate::errors::IndexedImageError::*;
use crate::file::FileType::Animated;
use crate::file::{verify_format, HEADER};
use crate::palette::FilePalette;
use crate::{palette, IciColor};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayType {
    /// Play from 0 to end once
    /// Must call [set_animate(true)] before the image will play
    /// Once it finishes will call [reset]
    Once,
    /// Play from end to 0 once
    /// Must call [set_animate(true)] before the image will play
    /// Once it finishes will call [reset]
    OnceReversed,
    /// Play from 0 to end repeatedly
    Loops,
    /// Play from end to 0 repeatedly
    LoopsReversed,
    /// Play from 0 to end to 0 repeatedly
    LoopsBoth,
}

impl PlayType {
    pub fn to_byte(&self) -> u8 {
        match self {
            Once => 0,
            OnceReversed => 1,
            Loops => 2,
            LoopsReversed => 3,
            LoopsBoth => 4,
        }
    }

    pub fn from_byte(value: u8) -> Option<PlayType> {
        match value {
            0 => Some(Once),
            1 => Some(OnceReversed),
            2 => Some(Loops),
            3 => Some(LoopsReversed),
            4 => Some(LoopsBoth),
            _ => None,
        }
    }
}

/// Series of images to play as an animation
///
/// # Usage
/// [set_animate] to play/pause
/// Call [update] in your UI/game update method, passing in your time step delta
#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedIndexedImage {
    width: u8,
    height: u8,
    per_frame: f64,
    palette: Vec<IciColor>,
    /// max allowed is 255
    frame_count: usize,
    frame_size: usize,
    pixels: Vec<u8>,
    current_frame: usize,
    next_frame_time: f64,
    highest_palette_idx: u8,
    animate: bool,
    play_type: PlayType,
    /// used with [LoopsBoth] to know whether increasing or decreasing
    loop_increasing: bool,
}

impl AnimatedIndexedImage {
    pub fn new(
        width: u8,
        height: u8,
        per_frame: f64,
        frame_count: u8,
        palette: Vec<IciColor>,
        pixels: Vec<u8>,
        play_type: PlayType,
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
        if per_frame < 0.0 {
            return Err(NegativePerFrame(per_frame));
        }
        let frame_size = width as usize * height as usize;
        if pixels.len() != frame_size * frame_count as usize {
            return Err(MissingData(pixels.len(), frame_size * frame_count as usize));
        }
        let highest_palette_idx = *pixels
            .iter()
            .max()
            .expect("Unable to get highest color index");
        let frame_size = width as usize * height as usize;
        let animate = matches!(
            play_type,
            PlayType::Loops | PlayType::LoopsReversed | PlayType::LoopsBoth
        );
        Ok(Self {
            width,
            height,
            per_frame,
            palette,
            pixels,
            current_frame: 0,
            next_frame_time: per_frame,
            highest_palette_idx,
            animate,
            frame_count: frame_count as usize,
            frame_size,
            play_type,
            loop_increasing: true,
        })
    }
}

impl AnimatedIndexedImage {
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

    /// This is unchecked and if color_idx > min_palette_size_supported
    /// will crash when rendering
    #[inline]
    pub fn set_pixel(
        &mut self,
        frame: u8,
        pixel_idx: usize,
        color_idx: u8,
    ) -> Result<(), IndexedImageError> {
        if frame >= self.frame_count as u8 {
            return Err(IndexOutOfRange(frame as usize, self.frame_count, "frames"));
        }
        if pixel_idx >= self.frame_size {
            return Err(IndexOutOfRange(pixel_idx, self.frame_size, "pixels"));
        }
        let idx = (frame as usize * self.frame_size) + pixel_idx;
        self.pixels[idx] = color_idx;
        Ok(())
    }

    #[inline]
    pub fn get_pixel(&self, frame: u8, pixel_idx: usize) -> Result<u8, IndexedImageError> {
        if frame >= self.frame_count as u8 {
            return Err(IndexOutOfRange(frame as usize, self.frame_count, "frames"));
        }
        if pixel_idx >= self.frame_size {
            return Err(IndexOutOfRange(pixel_idx, self.frame_size, "pixels"));
        }
        let idx = (frame as usize * self.frame_size) + pixel_idx;
        Ok(self.pixels[idx])
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

    #[inline]
    pub fn get_per_frame(&self) -> f64 {
        self.per_frame
    }

    #[inline]
    pub fn set_per_frame(&mut self, seconds: f64) {
        self.per_frame = seconds;
    }

    #[inline]
    pub fn set_animate(&mut self, animate: bool) {
        self.animate = animate;
    }

    #[inline]
    pub fn animate(&self) -> bool {
        self.animate
    }

    #[inline]
    pub fn frame_count(&self) -> u8 {
        self.frame_count as u8
    }

    /// Doesn't go to next frame until [update] is called
    #[inline]
    pub fn skip_to_next_frame(&mut self) {
        self.next_frame_time = -0.1;
    }

    /// Add `seconds` as one off dely for next frame
    #[inline]
    pub fn delay_next_frame(&mut self, seconds: f64) {
        self.next_frame_time += seconds;
    }

    #[inline]
    pub fn play_type(&self) -> PlayType {
        self.play_type
    }

    /// Frame timer to per frame and then depending on play type
    /// - Once - Frame to 0, playing to false
    /// - OnceReversed - Frame to end, playing to false
    /// - Looping - Frame to 0, playing to true
    /// - LoopingReversed - Frame to end, playing to true
    /// - LoopingBoth - Frame to 0, playing to true
    #[inline]
    pub fn reset(&mut self) {
        let (idx, animated) = match self.play_type {
            Once => (0, false),
            OnceReversed => (self.frame_count - 1, false),
            _ => (0, true),
        };
        self.current_frame = idx;
        self.animate = animated;
        self.next_frame_time = self.per_frame;
    }

    /// Sets play type and [reset]s
    pub fn set_play_type(&mut self, play_type: PlayType) {
        self.play_type = play_type;
        self.reset();
    }

    /// Like [set_play_type] but doesn't change anything else
    pub fn set_just_play_type(&mut self, play_type: PlayType) {
        self.play_type = play_type;
    }

    /// Changes play type {
    /// Once <-> OnceReversed
    /// Loops <-> LoopsReversed
    /// LoopsBoth swaps direction
    pub fn reverse(&mut self) {
        match self.play_type {
            Once => self.play_type = OnceReversed,
            OnceReversed => self.play_type = Once,
            Loops => self.play_type = LoopsReversed,
            LoopsReversed => self.play_type = Loops,
            LoopsBoth => self.loop_increasing = !self.loop_increasing,
        }
    }
}

impl AnimatedIndexedImage {
    /// Update frame timing
    ///
    /// * `delta` - Time delta, e.g. `timing.fixed_time_step`
    pub fn update(&mut self, delta: f64) {
        if self.animate {
            if self.next_frame_time < 0.0 {
                self.next_frame_time = self.per_frame;
                match self.play_type {
                    Once => {
                        self.current_frame += 1;
                        if self.current_frame >= self.frame_count {
                            self.reset();
                        }
                    }
                    OnceReversed => {
                        if self.current_frame > 0 {
                            self.current_frame -= 1;
                        } else {
                            self.reset();
                        }
                    }
                    Loops => {
                        self.current_frame += 1;
                        if self.current_frame >= self.frame_count {
                            self.current_frame = 0;
                        }
                    }
                    LoopsReversed => {
                        if self.current_frame > 0 {
                            self.current_frame -= 1;
                        } else {
                            self.current_frame = self.frame_count - 1;
                        }
                    }
                    LoopsBoth => {
                        if self.loop_increasing {
                            self.current_frame += 1;
                            if self.current_frame >= self.frame_count {
                                self.loop_increasing = false;
                                self.current_frame = self.frame_count - 1;
                            }
                        } else if self.current_frame > 0 {
                            self.current_frame -= 1;
                        } else {
                            self.loop_increasing = true;
                        }
                    }
                }
            }
            self.next_frame_time -= delta;
        }
    }
}

impl AnimatedIndexedImage {
    /// Errors will only be returned if you [FilePalette::Name] and the len is invalid
    pub fn to_file_contents(&self, palette: &FilePalette) -> Result<Vec<u8>, IndexedImageError> {
        let mut output = vec![];
        output.extend_from_slice(&HEADER);
        output.push(Animated.to_byte());

        palette::write(palette, self.get_palette(), &mut output)?;
        output.push(self.width);
        output.push(self.height);
        output.push(self.play_type.to_byte());
        output.push(self.frame_count as u8);
        output.extend_from_slice(&self.per_frame.to_be_bytes());
        output.extend_from_slice(&self.pixels);

        Ok(output)
    }

    /// Create an [AnimatedIndexedImage], image palette will be filled with transparency unless file contains colors
    /// use `image.set_palette*` to replace the palette
    pub fn from_file_contents(
        bytes: &[u8],
    ) -> Result<(AnimatedIndexedImage, FilePalette), IndexedImageError> {
        let file_type = verify_format(bytes)?;
        if file_type != Animated {
            return Err(InvalidFileFormat(
                0,
                format!(
                    "Expected Animated Image file but found {}",
                    file_type.name()
                ),
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
        let play_type_byte = bytes[start + 2];
        let play_type = PlayType::from_byte(play_type_byte);
        if play_type.is_none() {
            return Err(InvalidFileFormat(
                start + 2,
                format!("Unsupported play type: {play_type_byte}"),
            ));
        }
        let frame_count = bytes[start + 3];
        if frame_count == 0 {
            return Err(InvalidFileFormat(
                start + 3,
                "Image has no frames".to_string(),
            ));
        }
        let f64_bytes = &bytes[start + 4..=start + 11];
        let per_frame = f64::from_be_bytes([
            f64_bytes[0],
            f64_bytes[1],
            f64_bytes[2],
            f64_bytes[3],
            f64_bytes[4],
            f64_bytes[5],
            f64_bytes[6],
            f64_bytes[7],
        ]);
        if per_frame <= 0.0 {
            return Err(InvalidFileFormat(
                start + 4,
                format!("Per frame time is invalid: {per_frame}"),
            ));
        }
        let pixels_start = start + 12;
        let frame_size = width * height;
        let frame_pixel_count = frame_size as usize * frame_count as usize;
        if bytes.len() < pixels_start + frame_pixel_count {
            return Err(InvalidFileFormat(
                pixels_start,
                "Image has incomplete frame data".to_string(),
            ));
        }
        let pixels = &bytes[pixels_start..pixels_start + frame_pixel_count];

        let highest = *pixels.iter().max().expect("Invalid pixels data") as usize;
        let colors = match colors {
            None => vec![IciColor::transparent(); highest + 1],
            Some(colors) => colors,
        };

        AnimatedIndexedImage::new(
            width,
            height,
            per_frame,
            frame_count,
            colors,
            pixels.to_vec(),
            play_type.unwrap(),
        )
        .map(|image| (image, pal_type))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::palette::FilePalette::*;

    #[test]
    fn write_and_read_no_data() {
        let input = AnimatedIndexedImage::new(
            2,
            2,
            0.3,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2, 1, 2, 1, 0],
            Once,
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
                Animated.to_byte(),
                NoData.to_byte(),
                2,
                2,
                0,
                2,
                63,
                211,
                51,
                51,
                51,
                51,
                51,
                51,
                0,
                0,
                1,
                2,
                1,
                2,
                1,
                0
            ]
        );
        let (output, pal) = AnimatedIndexedImage::from_file_contents(&bytes).unwrap();
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
        let input = AnimatedIndexedImage::new(
            2,
            2,
            0.3,
            3,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2, 0, 0, 1, 2, 2, 1, 0, 0],
            OnceReversed,
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
                Animated.to_byte(),
                ID(0).to_byte(),
                0,
                15,
                2,
                2,
                1,
                3,
                63,
                211,
                51,
                51,
                51,
                51,
                51,
                51,
                0,
                0,
                1,
                2,
                0,
                0,
                1,
                2,
                2,
                1,
                0,
                0
            ]
        );
        let (output, pal) = AnimatedIndexedImage::from_file_contents(&bytes).unwrap();
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
        let input = AnimatedIndexedImage::new(
            2,
            2,
            0.3,
            2,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2, 1, 2, 1, 0],
            Loops,
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
                Animated.to_byte(),
                Name(String::new()).to_byte(),
                4,
                b'T',
                b'e',
                b's',
                b't',
                2,
                2,
                2,
                2,
                63,
                211,
                51,
                51,
                51,
                51,
                51,
                51,
                0,
                0,
                1,
                2,
                1,
                2,
                1,
                0
            ]
        );
        let (output, pal) = AnimatedIndexedImage::from_file_contents(&bytes).unwrap();
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
        let input = AnimatedIndexedImage::new(
            2,
            2,
            0.3,
            4,
            vec![
                IciColor::transparent(),
                IciColor::new(50, 51, 52, 53),
                IciColor::new(60, 61, 62, 63),
            ],
            vec![0, 0, 1, 2, 1, 2, 1, 0, 0, 0, 0, 1, 2, 1, 2, 1],
            LoopsBoth,
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
                Animated.to_byte(),
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
                4,
                4,
                63,
                211,
                51,
                51,
                51,
                51,
                51,
                51,
                0,
                0,
                1,
                2,
                1,
                2,
                1,
                0,
                0,
                0,
                0,
                1,
                2,
                1,
                2,
                1
            ]
        );
        let (output, pal) = AnimatedIndexedImage::from_file_contents(&bytes).unwrap();
        assert_eq!(input, output);
        assert_eq!(pal, Colors);
    }

    #[test]
    fn set_palette() {
        let image = AnimatedIndexedImage::new(
            3,
            3,
            0.3,
            2,
            vec![
                IciColor::new(255, 255, 255, 255),
                IciColor::new(0, 0, 0, 255),
            ],
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0],
            Once,
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
        let image = AnimatedIndexedImage::new(
            2,
            4,
            0.3,
            2,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0, 0, 1, 2, 2, 0, 1, 2, 0],
            Loops,
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
        let image = AnimatedIndexedImage::new(
            2,
            4,
            0.3,
            2,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0, 0, 1, 2, 2, 0, 1, 2, 0],
            Loops,
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
    fn pixel_accessors() {
        let mut image = AnimatedIndexedImage::new(
            2,
            4,
            0.3,
            2,
            vec![
                IciColor::new(1, 1, 1, 1),
                IciColor::new(2, 2, 2, 2),
                IciColor::new(3, 3, 3, 3),
            ],
            vec![0, 1, 2, 2, 0, 1, 2, 0, 1, 2, 1, 2, 0, 2, 1, 0],
            Loops,
        )
        .unwrap();
        let idx = image.get_pixel_index(1, 2).unwrap();
        assert!(image.get_pixel_index(4, 4).is_err());
        assert_eq!(image.get_pixel(1, idx).unwrap(), 2);
        assert!(image.set_pixel(1, idx, 0).is_ok());
        assert_eq!(image.get_pixel(1, idx).unwrap(), 0);
        assert_eq!(image.get_pixel(0, idx).unwrap(), 1);
        assert!(image.set_pixel(0, idx, 2).is_ok());
        assert_eq!(image.get_pixel(0, idx).unwrap(), 2);
    }
}
