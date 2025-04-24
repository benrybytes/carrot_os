use core::fmt::Write;

use lazy_static::lazy_static;
use spin::Mutex; // who controls what piece of data, continuous sleeping
use volatile::Volatile; // no compiler optimizations lol

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {

    // use volatile pointer to make compiler aware
    // of text side effects than to ignore VGA buffer existence
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// writer should contain the information
// it currently is and the buffer it references to write
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

// use formatting macros by allowing all types of string
// types and writing to our buffer after Write trait handles it
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_color = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_color
                });

                self.column_position += 1;
            }
        }
    }

    // move characters up one row, and finish with new line, clear
    // previous row where characters were
    pub fn new_line(&mut self) {

        // we want to get the row on the top, not -1
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {

                // get character at current row and column
                let character = self.buffer.chars[row][col].read(); // get a copy

                // we decrement one to put the characters on top
                // then, we create new line, moving our writer
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // eliminate bottom row of characters
        self.clear_row(BUFFER_HEIGHT - 1);

        // start from zero as a new line was made
        self.column_position = 0;
    }

    // clear the row with empty characters
    pub fn clear_row(&mut self, row: usize) {
        let blank_character = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank_character);
        }

    }

    pub fn write_string(&mut self, s: &str) {

        // iterate to the string and convert to readable bytes
        for byte in s.bytes() {
            match byte {
                0x20..0x7e | b'\n' => self.write_byte(byte),

                // non ASCII characters
                _ => self.write_byte(0xfe)
            }
        }
    }

}

// allow access of static globally when accessed the first time to not make compiler complain
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn _print_value() {
    let mut writer = Writer {
        column_position: 44, 
        color_code: ColorCode::new(Color::Yellow, Color::Black),

        // create a reference to a pointer available
        // for the whole program and won't get dropped
        // until program ends and no function can
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}
    };

    writer.write_string("testing");
    writer.write_string("Wörld!");
    write!(writer, "favorite numbers: {}", 42.0).unwrap();
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write; // implemented
    // uses is when invoking write! macro and this print is called
    // calls our write_str method from Writer implementation of Write
    WRITER.lock().write_fmt(args).unwrap();
}


#[macro_export]
macro_rules! print {
    // calls _print when printing arguments
    ($($arg:tt)*) => ($crate::vga_text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));

    // format the arguments for the current print implementation
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}


#[test_case]
fn test_println_simple() {
    println!("testing print");
}

#[test_case]
fn test_println_alot() {
    for _ in 0..200 {
        println!("listen you");
    }
}

#[test_case]
fn test_println_output() {
    let s = "rust is pretty cool";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        // get a copy of the rendered character
        let rendered_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(rendered_char.ascii_character), c);
    }
}
