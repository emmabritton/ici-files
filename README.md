# ICI Files

Encodes and decodes ICI files and JASC palettes

Designed to be used with [Buffer Graphics](https://github.com/emmabritton/buffer-graphics-lib), and in turn [Pixel Graphics](https://github.com/emmabritton/pixel-graphics-lib).

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

#### Single

Single static image, max width and height is 255

#### Animated

Multi frame image, max width, height and frame count is 255.
Also contains a frame rate as fractional seconds per frame.
All frames must be the same size.