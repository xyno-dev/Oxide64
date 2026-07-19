#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod graphics;
mod interrupts;
mod serial;

use core::panic::PanicInfo;
use multiboot2::{BootInformation, BootInformationHeader};

use graphics::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printerr!("{info}");
    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

pub fn init() {
    interrupts::init_idt();
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

    init();

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
