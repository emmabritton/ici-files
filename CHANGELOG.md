# Changelog

### Version 0.3.0
### Breaking
- Remove `Into`/`From` implementations for `Color`
- Remove specific conversions method (such as `_as_f32_array`)
- Add `to_rgb`, `from_rgb`, `to_argb`, `from_argb`, `to_rgba`, `from_rgba` for `u32`, `(u8,u8,u8,u8)`, `[u8;4]`, `(u8,u8,u8)`, `[u8;3]`, `(f32,f32,f32,f32)`, `[f32;4]`, `(f32,f32,f32)`, `[f32;3]`
  - This was as it was confusing which format the conversion methods worked with
- Migration:
  - For `let num: u32 = color.into()` use `let num: u32 = color.to_rgba()` (or `to_argb()` as needed)
  - For `let arr = color.as_f32_array()` use `let arr: [f32; 4] = color.to_rgba()`
  - For `let color: Color = 1_u32.into()` use `let color = Color::from_rgba(1_u32)`

### Version 0.2.5
- Update docs
- Update deps

### Version 0.2.4
- Update deps
- Add to/from u32 color conversion

### Version 0.2.3
- Fix bug where minimum palette size didn't increase when setting pixels

### Version 0.2.2
- Add with_red, etc methods to color

### Version 0.2.1
- Make new image methods public

### Version 0.2.0
### Breaking
- Replace `IciColor` with `Color`
- Add flip, rotate and recolor methods to `IndexedImage`
- Add various `_unchecked` methods and remove unsafe code from non unsafe methods

### Version 0.1.7
- Add `JascPalette`

### Version 0.1.6
- Add `IndexedWrapper` 

### Version 0.1.5
- Fix crash with large palette sizes

### Version 0.1.4
- Add `width()` and `height()` for `AnimatedIndexedImage`

### Version 0.1.3
- Add `get_frame(idx)` and `as_images()` for `AnimatedIndexedImage`

### Version 0.1.2
- Add `width()` and `height()` for `IndexedImage`

### Version 0.1.1
- Add `is_transparent()` for `IciColor`

### Version 0.1.0
- Initial release