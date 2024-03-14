use crate::errors::IndexedImageError;
use crate::errors::IndexedImageError::{InvalidScaleParams, TooBigPostScale};
use crate::image::IndexedImage;
use crate::scaling::Scaling::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Scaling {
    /// Increase size of image by x_scale and y_scale
    /// Where {2,2} doubles the size
    NearestNeighbour {
        x_scale: NonZeroUsize,
        y_scale: NonZeroUsize,
    },
    Epx2x,
    Epx4x,
}

impl Scaling {
    pub fn nearest_neighbour(x: usize, y: usize) -> Result<Scaling, IndexedImageError> {
        Ok(NearestNeighbour {
            x_scale: NonZeroUsize::new(x).ok_or(InvalidScaleParams(x, y))?,
            y_scale: NonZeroUsize::new(y).ok_or(InvalidScaleParams(x, y))?,
        })
    }

    /// Double image size using nearest neighbour
    pub fn nn_double() -> Scaling {
        NearestNeighbour {
            x_scale: NonZeroUsize::new(2).expect("2 is 0?"),
            y_scale: NonZeroUsize::new(2).expect("2 is 0?"),
        }
    }
}

pub(crate) fn scale_nearest_neighbor(
    image: &IndexedImage,
    x_scale: usize,
    y_scale: usize,
) -> Result<IndexedImage, IndexedImageError> {
    let new_width = image.width() as usize * x_scale;
    let new_height = image.height() as usize * y_scale;
    if new_height > 255 || new_width > 255 {
        return Err(TooBigPostScale(new_width, new_height));
    }
    let new_width = new_width as u8;
    let new_height = new_height as u8;
    let mut new_image = IndexedImage::blank(new_width, new_height, image.get_palette().to_vec());
    let x_scale = 1.0 / x_scale as f32;
    let y_scale = 1.0 / y_scale as f32;
    for y in 0..new_height {
        for x in 0..new_width {
            let px = (x as f32 * x_scale).floor() as u8;
            let py = (y as f32 * y_scale).floor() as u8;
            let target_i = new_image.get_pixel_index(x, y)?;
            let pi = image.get_pixel_index(px, py)?;
            new_image.set_pixel(target_i, image.get_pixel(pi)?)?;
        }
    }
    Ok(new_image)
}

pub(crate) unsafe fn scale_nearest_neighbor_unchecked(
    image: &IndexedImage,
    x_scale: usize,
    y_scale: usize,
) -> IndexedImage {
    let new_width = (image.width() as usize * x_scale) as u8;
    let new_height = (image.height() as usize * y_scale) as u8;
    let mut new_image = IndexedImage::blank(new_width, new_height, image.get_palette().to_vec());
    let x_scale = 1.0 / x_scale as f32;
    let y_scale = 1.0 / y_scale as f32;
    for y in 0..new_height {
        for x in 0..new_width {
            let px = (x as f32 * x_scale).floor() as u8;
            let py = (y as f32 * y_scale).floor() as u8;
            let target_i = new_image.get_pixel_index_unchecked(x, y);
            let pi = image.get_pixel_index_unchecked(px, py);
            new_image.set_pixel_unchecked(target_i, image.get_pixel_unchecked(pi));
        }
    }
    new_image
}

pub(crate) fn scale_epx(image: &IndexedImage) -> Result<IndexedImage, IndexedImageError> {
    let new_width = image.width() as usize * 2;
    let new_height = image.height() as usize * 2;
    if new_height > 255 || new_width > 255 {
        return Err(TooBigPostScale(new_width, new_height));
    }
    let new_width = new_width as u8;
    let new_height = new_height as u8;
    let mut new_image = IndexedImage::blank(new_width, new_height, image.get_palette().to_vec());
    for x in 0..image.width() {
        for y in 0..image.height() {
            let mut p1 = image.get_pixel(image.get_pixel_index(x, y)?)?;
            let mut p2 = p1;
            let mut p3 = p1;
            let mut p4 = p1;
            let a = image.get_pixel(image.get_pixel_index(x, if y > 0 { y - 1 } else { y })?)?;
            let c = image.get_pixel(image.get_pixel_index(if x > 0 { x - 1 } else { x }, y)?)?;
            let b = image.get_pixel(
                image.get_pixel_index(if x < image.width() - 2 { x + 1 } else { x }, y)?,
            )?;
            let d = image.get_pixel(
                image.get_pixel_index(x, if y < image.height() - 2 { y + 1 } else { y })?,
            )?;

            if c == a && c != d && a != b {
                p1 = a
            }
            if a == b && a != c && b != d {
                p2 = b
            }
            if d == c && d != b && c != a {
                p3 = c
            }
            if b == d && b != a && d != c {
                p4 = d
            }

            let nx = x * 2;
            let ny = y * 2;
            new_image.set_pixel(new_image.get_pixel_index(nx, ny)?, p1)?;
            new_image.set_pixel(new_image.get_pixel_index(nx + 1, ny)?, p2)?;
            new_image.set_pixel(new_image.get_pixel_index(nx, ny + 1)?, p3)?;
            new_image.set_pixel(new_image.get_pixel_index(nx + 1, ny + 1)?, p4)?;
        }
    }
    Ok(new_image)
}

pub(crate) unsafe fn scale_epx_unchecked(image: &IndexedImage) -> IndexedImage {
    let new_width = (image.width() as usize * 2) as u8;
    let new_height = (image.height() as usize * 2) as u8;
    let mut new_image = IndexedImage::blank(new_width, new_height, image.get_palette().to_vec());
    for x in 0..image.width() {
        for y in 0..image.height() {
            let mut p1 = image.get_pixel_unchecked(image.get_pixel_index_unchecked(x, y));
            let mut p2 = p1;
            let mut p3 = p1;
            let mut p4 = p1;
            let a = image.get_pixel_unchecked(
                image.get_pixel_index_unchecked(x, if y > 0 { y - 1 } else { y }),
            );
            let c = image.get_pixel_unchecked(
                image.get_pixel_index_unchecked(if x > 0 { x - 1 } else { x }, y),
            );
            let b = image.get_pixel_unchecked(
                image.get_pixel_index_unchecked(if x < image.width() - 2 { x + 1 } else { x }, y),
            );
            let d = image.get_pixel_unchecked(
                image.get_pixel_index_unchecked(x, if y < image.height() - 2 { y + 1 } else { y }),
            );

            if c == a && c != d && a != b {
                p1 = a
            }
            if a == b && a != c && b != d {
                p2 = b
            }
            if d == c && d != b && c != a {
                p3 = c
            }
            if b == d && b != a && d != c {
                p4 = d
            }

            let nx = x * 2;
            let ny = y * 2;
            new_image.set_pixel_unchecked(new_image.get_pixel_index_unchecked(nx, ny), p1);
            new_image.set_pixel_unchecked(new_image.get_pixel_index_unchecked(nx + 1, ny), p2);
            new_image.set_pixel_unchecked(new_image.get_pixel_index_unchecked(nx, ny + 1), p3);
            new_image.set_pixel_unchecked(new_image.get_pixel_index_unchecked(nx + 1, ny + 1), p4);
        }
    }
    new_image
}
