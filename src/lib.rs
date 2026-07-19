#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

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

use core::arch::asm;

fn debug_byte(b: u8) {
    unsafe {
        asm!("out dx, al", in("dx") 0x402u16, in("al") b);
    }
}

fn debug_str(s: &str) {
    for b in s.bytes() {
        debug_byte(b);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::RED);
    let mut message: String<256> = String::new();
    write!(message, "{info}").unwrap();
    debug_str(message.as_str());
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

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
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
            text_buffer: [[b' '; graphics::COLS]; graphics::ROWS],
            column: 0,
        });
    }

    #[cfg(test)]
    test_main();

    #[cfg(test)]
    println!("SUCCESS");

    loop {}
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[PASS]");
}
