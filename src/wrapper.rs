use crate::errors::IndexedImageError;
use crate::image::IndexedImage;
use crate::prelude::AnimatedIndexedImage;
use crate::IciColor;

#[derive(Debug, Clone, PartialEq)]
pub enum IndexedWrapper {
    Static(IndexedImage),
    Animated(AnimatedIndexedImage),
}

impl Into<IndexedWrapper> for IndexedImage {
    fn into(self) -> IndexedWrapper {
        IndexedWrapper::Static(self)
    }
}

impl Into<IndexedWrapper> for AnimatedIndexedImage {
    fn into(self) -> IndexedWrapper {
        IndexedWrapper::Animated(self)
    }
}

impl IndexedWrapper {
    /// Replace palette for image
    /// Will only return an error if the new palette has less colors than the image needs
    pub fn set_palette(&mut self, palette: &[IciColor]) -> Result<(), IndexedImageError> {
        match self {
            IndexedWrapper::Static(img) => img.set_palette(palette),
            IndexedWrapper::Animated(img) => img.set_palette(palette),
        }
    }

    /// Replace palette for image, any pixels outside the new palette will be replaced with `id`
    /// Will only return an error if id is outside the new palette
    pub fn set_palette_replace_id(
        &mut self,
        palette: &[IciColor],
        id: u8,
    ) -> Result<(), IndexedImageError> {
        match self {
            IndexedWrapper::Static(img) => img.set_palette_replace_id(palette, id),
            IndexedWrapper::Animated(img) => img.set_palette_replace_id(palette, id),
        }
    }

    /// Replace palette for image, any color indexes outside the palette will be expanded with `color`
    pub fn set_palette_replace_color<C: Into<IciColor> + Copy>(
        &mut self,
        palette: &[IciColor],
        color: C,
    ) {
        match self {
            IndexedWrapper::Static(img) => img.set_palette_replace_color(palette, color),
            IndexedWrapper::Animated(img) => img.set_palette_replace_color(palette, color),
        }
    }

    pub fn size(&self) -> (u8, u8) {
        match self {
            IndexedWrapper::Static(img) => img.size(),
            IndexedWrapper::Animated(img) => img.size(),
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        match self {
            IndexedWrapper::Static(img) => img.get_pixels(),
            IndexedWrapper::Animated(img) => img.get_pixels(),
        }
    }

    pub fn get_pixel_index(&self, x: u8, y: u8) -> Result<usize, IndexedImageError> {
        match self {
            IndexedWrapper::Static(img) => img.get_pixel_index(x, y),
            IndexedWrapper::Animated(img) => img.get_pixel_index(x, y),
        }
    }

    pub fn get_color(&self, idx: u8) -> Result<IciColor, IndexedImageError> {
        match self {
            IndexedWrapper::Static(img) => img.get_color(idx),
            IndexedWrapper::Animated(img) => img.get_color(idx),
        }
    }

    pub fn set_color(&mut self, idx: u8, color: IciColor) -> Result<(), IndexedImageError> {
        match self {
            IndexedWrapper::Static(img) => img.set_color(idx, color),
            IndexedWrapper::Animated(img) => img.set_color(idx, color),
        }
    }

    pub fn get_palette(&self) -> &[IciColor] {
        match self {
            IndexedWrapper::Static(img) => img.get_palette(),
            IndexedWrapper::Animated(img) => img.get_palette(),
        }
    }

    pub fn min_palette_size_supported(&self) -> u8 {
        match self {
            IndexedWrapper::Static(img) => img.min_palette_size_supported(),
            IndexedWrapper::Animated(img) => img.min_palette_size_supported(),
        }
    }

    pub fn width(&self) -> u8 {
        match self {
            IndexedWrapper::Static(img) => img.width(),
            IndexedWrapper::Animated(img) => img.width(),
        }
    }

    pub fn height(&self) -> u8 {
        match self {
            IndexedWrapper::Static(img) => img.height(),
            IndexedWrapper::Animated(img) => img.height(),
        }
    }

    pub fn update(&mut self, delta: f64) {
        match self {
            IndexedWrapper::Static(_) => {}
            IndexedWrapper::Animated(img) => img.update(delta),
        }
    }

    pub fn reset(&mut self) {
        match self {
            IndexedWrapper::Static(_) => {}
            IndexedWrapper::Animated(img) => img.reset(),
        }
    }

    #[inline]
    pub fn set_animate(&mut self, animate: bool) {
        match self {
            IndexedWrapper::Static(_) => {}
            IndexedWrapper::Animated(img) => img.set_animate(animate),
        }
    }

    #[inline]
    pub fn animating(&self) -> bool {
        match self {
            IndexedWrapper::Static(_) => false,
            IndexedWrapper::Animated(img) => img.animating(),
        }
    }

    pub fn frame_count(&self) -> u8 {
        match self {
            IndexedWrapper::Static(_) => 1,
            IndexedWrapper::Animated(img) => img.frame_count(),
        }
    }

    pub fn is_animation(&self) -> bool {
        matches!(self, IndexedWrapper::Animated(_))
    }
}
