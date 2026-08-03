use heapless::{String, Vec};

use crate::{fat, interrupts, print, println};

fn ls() {
    let bpb = fat::read_bpb();
    let directories = fat::list_directories(&bpb);
    println!("Name        Size");
    println!("----        ----");
    for directory in directories {
        let name = str::from_utf8(&directory.name).unwrap_or("-------").trim();
        let extension = str::from_utf8(&directory.extension).unwrap_or("---").trim();
        let size = directory.size;

        println!("{name}.{extension} {size:>pad$} bytes", pad = 12 - (name.len() + extension.len()));
    }
}

pub fn shell_loop() -> ! {
    loop {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut buffer = interrupts::KEYBOARD_BUF.lock();
            if buffer.contains(&'\n') {
                buffer.pop();
                let mut command: String<256> = String::new();
                for c in (*buffer).iter() {
                    command.push(*c).unwrap();
                }
                match command.as_str() {
                    "ls" => ls(),
                    command if command.trim().is_empty() => {},
                    command => println!("{command}: command not found")
                }
                print!("$ ");
                *buffer = Vec::new();
            }
        });
        x86_64::instructions::hlt();
    }
}
