use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Baseline, Text, renderer::TextRenderer},
};

use x86_64::instructions::interrupts;

use heapless::String;
use spin::Mutex;

use core::fmt;
use core::fmt::Write;

use crate::serial;

pub static FRAME_BUFFER: Mutex<Option<FrameBuffer>> = Mutex::new(None);
pub static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

pub const ROWS: usize = 768 / 11;
pub const COLS: usize = 1024 / 7;

pub struct FrameBuffer {
    pub buffer: &'static mut [u8],
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
}

impl DrawTarget for FrameBuffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x.is_negative() || coord.y.is_negative() {
                continue;
            }
            let idx = coord.y as usize * self.pitch + coord.x as usize * 4;
            if idx + 2 >= self.buffer.len() {
                continue;
            }
            self.buffer[idx] = color.b();
            self.buffer[idx + 1] = color.g();
            self.buffer[idx + 2] = color.r();
        }
        Ok(())
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

pub struct Writer {
    pub text_buffer: [[u8; COLS]; ROWS],
    pub column: usize,
}

impl Writer {
    fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => {
                self.newline();
            }
            b => {
                if self.column >= COLS {
                    self.newline();
                }
                self.text_buffer[ROWS - 1][self.column] = b;
                self.column += 1;
            }
        }
    }

    fn newline(&mut self) {
        for row in 0..(ROWS - 1) {
            self.text_buffer[row] = self.text_buffer[row + 1];
        }
        self.text_buffer[ROWS - 1] = [b' '; COLS];
        self.column = 0;
    }

    fn draw(&mut self, fb: &mut FrameBuffer) {
        for (i, row) in self.text_buffer.iter().enumerate() {
            let s: &str = core::str::from_utf8(row).unwrap();
            let style = MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(Rgb888::WHITE)
                .background_color(Rgb888::BLACK)
                .build();
            Text::new(s, Point::new(0, (i * 11) as i32), style)
                .draw(fb)
                .unwrap();
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for c in s.bytes() {
            self.write_byte(c);
        }
        self.draw(FRAME_BUFFER.lock().as_mut().unwrap());

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::graphics::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printerr {
    ($($arg:tt)*) => ($crate::graphics::_printerr(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    interrupts::without_interrupts(|| {
        if let Some(writer) = WRITER.lock().as_mut() {
            writer.write_fmt(args).unwrap();
        }
    });
}

pub fn _printerr(args: fmt::Arguments) {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::RED);
    let mut message: String<1024> = String::new();
    write!(message, "{args}").unwrap();
    let mut max_width = 0u32;
    let mut total_height = 0u32;
    for line in message.as_str().split('\n') {
        let metrics = style.measure_string(line, Point::zero(), Baseline::Alphabetic);
        max_width = max_width.max(metrics.bounding_box.size.width);
        total_height += metrics.bounding_box.size.height;
    }
    serial::debug_str(message.as_str());
    interrupts::without_interrupts(|| {
        if let Some(frame_buffer) = FRAME_BUFFER.lock().as_mut() {
            Text::new(
                &message,
                Point::new(
                    (1024 / 2) - (max_width as i32 / 2),
                    (768 / 2) - (total_height as i32 / 2),
                ),
                style,
            )
            .draw(frame_buffer)
            .unwrap();
        }
    });
}

#[test_case]
fn test_println_once() {
    print!("test_println_once... ");
    println!("[PASS]")
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        print!("test_println_many... ");
        println!("[PASS]")
    }
}

#[test_case]
fn test_println_output() {
    let s = "test_println_output... ";
    interrupts::without_interrupts(|| {
        println!();
        print!("{}", s);
        for (i, c) in s.bytes().enumerate() {
            let screen_char = WRITER.lock().as_mut().unwrap().text_buffer[ROWS - 1][i];
            assert_eq!(screen_char, c);
        }
    });
    println!("[PASS]")
}
