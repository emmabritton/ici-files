[![Crates.io](https://img.shields.io/crates/v/ici-files)](https://crates.io/crates/ici-files "Crates.io version")
[![Documentation](https://img.shields.io/docsrs/ici-files)](https://docs.rs/ici-files "Documentation")


# ICI Files

## Usage

```toml
ici-files = "0.2.1"
```

Encodes and decodes ICI files and JASC palettes

Designed to be used with [Buffer Graphics](https://github.com/emmabritton/ici-files), and in turn [Pixel Graphics](https://github.com/emmabritton/pixel-graphics-lib).

Indexed Color Images come in two forms:
1. Single
2. Animated

All three may contain palette data in one of these forms:
1. No palette data
2. Palette ID (u16)
3. Palette Name (String 1..=255)
4. Palette Colours (RGBA 1..=255)

## Palettes

#### No Data

The file doesn't contain any palette information. The `Image` struct will default to a palette of the correct size but filled with transparency.
Use `Image::with_palette` to set the palette or `Image::set_color` to set a specific color.

#### ID
 
The file has a palette ID between 0..=65535. The `Image` struct will default to a palette of the correct size but filled with transparency.
Use `Image::with_palette` to set the palette or `Image::set_color` to set a specific color.

#### Name

The file has a UTF-8 palette name, between 1..=255 bytes long. The `Image` struct will default to a palette of the correct size but filled with transparency.
Use `Image::with_palette` to set the palette or `Image::set_color` to set a specific color.

#### Colors

The file contains a list of RGBA colors.

## Image formats

### Single

Single static image, max width and height is 255

### Animated

Multi frame image, max width, height and frame count is 255.
Also contains a frame rate as fractional seconds per frame.
All frames must be the same size.

#### IndexedWrapper

Stores either a static or animated image and provides a limited abstract interface

## Features

> Default: `serde`

#### Serde

Adds serialize and deserialize to some structs