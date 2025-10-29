
# VGA Text Buffer

## Color

`#[allow(dead_code)]`: clears warnings on variants not being used
`#[repr(u8)]`: attribute allowing all variants to be u8
```rs
// in src/vga_buffer.rs

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}
```
### representing colors

background, we shift it four bits and fill those with 0000, as background is in 12-14 bits
foreground, is in bits 8-11, so we shift from 8 to 12, while 0-7 are ASCII

```rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // allows this tuple structure to be formatted as same size as fields it holds
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}
```

## Text buffer

we make a buffer to hold the basic layout a screen holds of number of characters
```rs
// in src/vga_buffer.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // represent unknown size structure to known via reading using C and correct field ordering of bits in C struct
struct ScreenChar {
    ascii_character: u8, // first 0-7 bits
    color_code: ColorCode, // rest of bits
}

// align with screen
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT], // make 80 x 25 number of screen chars buffer holds for screen
}
```

### writer

allows us to know where to write the text 

we want the static to have a lifetime for the whole duration of the program

```rs
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}
```
