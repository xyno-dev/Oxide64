#![no_std]
#![no_main]

mod graphics;

use core::panic::PanicInfo;

use multiboot2::{BootInformation, BootInformationHeader};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Text},
};

use core::fmt::Write;
use graphics::*;
use heapless::String;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::RED);
    let mut message: String<256> = String::new();
    write!(message, "{info}").unwrap();
    if let Some(frame_buffer) = FRAME_BUFFER.lock().as_mut() {
        Text::with_alignment(
            &message,
            Point::new(1024 / 2, 768 / 2),
            style,
            Alignment::Center,
        )
        .draw(frame_buffer)
        .unwrap();
    }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) -> ! {
    let boot_info = unsafe {
        BootInformation::load(multiboot_info_ptr as *const BootInformationHeader)
            .expect("Failed to parse Multiboot2 structure")
    };

    if let Some(fb_tag) = boot_info.framebuffer_tag() {
        let fb_tag = fb_tag.unwrap();

        let fb_address = fb_tag.address();

        let width = fb_tag.width();
        let height = fb_tag.height();
        let pitch = fb_tag.pitch();
        // let bpp = fb_tag.bpp();

        let total_bytes = (height * pitch) as usize;
        let buffer = unsafe { core::slice::from_raw_parts_mut(fb_address as *mut u8, total_bytes) };

        *FRAME_BUFFER.lock() = Some(FrameBuffer {
            buffer,
            width: width as usize,
            height: height as usize,
            pitch: pitch as usize,
        });

        *WRITER.lock() = Some(Writer {
            text_buffer: [[b' '; 1024 / 7]; 768 / 11],
            column: 0,
        });
    }
    let mut x: u32 = 25;
    loop {
        x -= 1;
        println!("Performing 10 / {x}\n");
        let _ = 10 / x;
    }
}
