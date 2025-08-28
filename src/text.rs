use limine::request::FramebufferRequest;
use limine::response::FramebufferResponse;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex; // who controls what piece of data, continuous sleeping
use volatile::Volatile; // no compiler optimizations lol
use limine::framebuffer::Framebuffer;


#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[repr(C)]
struct Psf1Header {
    magic: u16,
    mode: u8,
    charsize: u8,
}

extern "C" {
    static _binary_Cyr_a8x16_psf_start: u8;
    static _binary_Cyr_a8x16_psf_end: u8;
}

fn font_data() -> &'static [u8] {
    unsafe {
        let start = &_binary_Cyr_a8x16_psf_start as *const u8 as usize;
        let end = &_binary_Cyr_a8x16_psf_end as *const u8 as usize;
        core::slice::from_raw_parts(start as *const u8, end - start)
    }
}

pub struct Font {
    header: &'static Psf1Header,
    glyphs: &'static [u8],
}

impl Font {
    pub fn load() -> Self {
        let data = font_data();
        let header = unsafe { &*(data.as_ptr() as *const Psf1Header) };
        let glyphs = &data[core::mem::size_of::<Psf1Header>()..];

        Font { header, glyphs }
    }

    pub fn glyph(&self, c: u8) -> &[u8] {
        let chars = if self.header.mode & 1 != 0 { 512 } else { 256 };
        let size = self.header.charsize as usize;
        let idx = (c as usize % chars) * size;
        &self.glyphs[idx..idx + size]
    }
}

pub struct Writer<'a> {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
    foreground: u32,
    background: u32,
    framebuffer: Framebuffer<'a>,
    font: Font,
}

impl Write for Writer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


                // for y in 0..height {
                //     if y % 16 == 0 {
                //         for x in 0..width {
                //             if x % 8 == 0 {
                //                 self.draw_char(0x20);
                //             }
                //         }
                //     }
                // }

impl Writer<'_> {
    fn load(foreground: Color, background: Color) -> Self {
        let framebuffer_value: Option<Framebuffer<'_>>;
        if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
            if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
                framebuffer_value = Some(framebuffer);
            } else {
                framebuffer_value = None;
            }
        } else {
            framebuffer_value = None;
        }

        match framebuffer_value {
            Some(framebuffer) => {
                let font = Font::load();
                let height = framebuffer.height();
                let width = framebuffer.width();
                Writer {
                    x: 0,
                    y: 0,
                    width,
                    height,
                    foreground: foreground as u32,
                    background: background as u32,
                    framebuffer,
                    font
                }
            }
            None => panic!{"could not"}
        }
    }
    pub fn new_line(&mut self) {
        // start from zero as a new line was made
        self.x = 0;
        self.y += 1;
        if self.y >= self.height {
            
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.x >= self.width {
                    self.new_line();
                }
                self.draw_char(byte);
                self.x +=1;
            }
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
    fn draw_char(&self, c: u8) {
        let glyph = self.font.glyph(c);

        for (row, byte) in glyph.iter().enumerate() {
            for bit in 0..8 {
                let pixel = if byte & (0x80 >> bit) != 0 { self.foreground } else { self.background };
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.

                let pixel_offset = (self.y * 16 + row as u64) * self.framebuffer.pitch() + (self.x * 8 + bit) * 4;
                unsafe {
                    self.framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(pixel)
                };

            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum Color {
    White = 0xc9ccca,
    Black = 0x222623
}

// allow access of static globally when accessed the first time to not make compiler complain
lazy_static! {
    pub static ref WRITER: Mutex<Writer<'static>> = Mutex::new(Writer::load(
        Color::White, 
        Color::Black
    ));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;   // new

    interrupts::without_interrupts(|| {     // new
        WRITER.lock().write_fmt(args).unwrap();
    });
}


#[macro_export]
macro_rules! print {
    // calls _print when printing arguments
    ($($arg:tt)*) => ($crate::text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));

    // format the arguments for the current print implementation
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

