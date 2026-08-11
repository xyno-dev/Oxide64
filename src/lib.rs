#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod fat;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod shell;
pub mod speaker;
pub mod time;

use core::panic::PanicInfo;
use multiboot2::{BootInformation, BootInformationHeader, MemoryMapTag};

use crate::memory::FrameAllocator;

use graphics::*;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printerr!("{info}");
    hlt_loop();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe {
        interrupts::disable_apic();
        interrupts::PICS.lock().initialize()
    };
    x86_64::instructions::interrupts::enable();
    time::init_pit_channel_0();
    speaker::init_pit_channel_2();
    fat::init();
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) -> ! {
    let boot_info: BootInformation = unsafe {
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

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "    start: 0x{:x}, length: 0x{:x}",
            area.start_address(),
            area.size()
        );
    }
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");
    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!(
            "    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.sh_addr, section.sh_size, section.sh_flags
        );
    }

    let kernel_start = elf_sections_tag
        .sections()
        .map(|s| s.sh_addr)
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag
        .sections()
        .map(|s| s.sh_addr + s.sh_size)
        .max()
        .unwrap();

    let multiboot_start = multiboot_info_ptr;
    let multiboot_end = multiboot_start + (boot_info.total_size() as usize);

    let mut frame_allocator = memory::arena_frame_allocator::ArenaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas().iter(),
    );

    let mut i = 0;
    while let Some(_) = frame_allocator.allocate_frame() {
        i += 1;
    }
    println!("allocated {} frames", i);

    println!("kernel_start: 0x{:x}", kernel_start);
    println!("kernel_end: 0x{:x}", kernel_end);
    println!("multiboot_start: 0x{:x}", multiboot_start);
    println!("multiboot_end: 0x{:x}", multiboot_end);

    init();

    graphics::splash();
    time::sleep(250);
    speaker::beep(261.63, 350);
    speaker::beep(329.63, 350);
    speaker::beep(392.00, 350);
    speaker::beep(523.25, 500);
    time::sleep(1000);
    graphics::clear_framebuffer();

    #[cfg(test)]
    test_main();

    #[cfg(test)]
    println!("SUCCESS");

    println!("Welcome to Oxide64!");
    speaker::beep(800.0, 50);
    time::sleep(1000);
    println!("You are currently in Ring 0. Not like you can do anything with it.");
    speaker::beep(800.0, 50);
    time::sleep(1000);
    println!("There is no shell, nor any way to interact with the file system.");
    speaker::beep(800.0, 50);
    time::sleep(1000);
    println!("You may type as if this were a text editor.");
    speaker::beep(800.0, 50);
    time::sleep(1000);
    println!("Have fun!");
    speaker::beep(800.0, 50);

    shell::shell_loop();
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[PASS]");
}
