use core::str::SplitWhitespace;

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

        println!(
            "{name}.{extension} {size:>pad$} bytes",
            pad = 12 - (name.len() + extension.len())
        );
    }
}

fn cat(filename: &str) {
    let bpb = fat::read_bpb();
    let directories = fat::list_directories(&bpb);
    let mut file_entry = None;

    for directory in directories {
        let name = str::from_utf8(&directory.name).unwrap_or("").trim();
        if name == filename {
            file_entry = Some(directory)
        }
    }

    if let Some(entry) = file_entry {
        let mut filestream = fat::FileStream::new(entry);
        while let Some(s) = filestream.read_line() {
            println!("{s}");
        }
    } else {
        println!("cat: {filename}: file not found")
    }
}

fn echo(mut s: SplitWhitespace) {
    while let Some(s) = s.next() {
        print!("{s} ");
    }
    print!("\x08\n");
}

pub fn shell_loop() -> ! {
    print!("# ");
    loop {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut buffer = interrupts::KEYBOARD_BUF.lock();
            if buffer.contains(&'\n') {
                buffer.pop();
                let mut command: String<256> = String::new();
                for c in (*buffer).iter() {
                    command.push(*c).unwrap();
                }
                let mut argc = command.split_whitespace();
                match argc.next().unwrap_or("") {
                    "ls" => ls(),
                    "cat" => cat(argc.next().unwrap_or_else(|| {
                        println!("cat: expected 2 arguments");
                        ""
                    })),
                    "echo" => echo(argc),
                    command if command.is_empty() => {}
                    command => println!("{command}: command not found"),
                }
                print!("# ");
                *buffer = Vec::new();
            }
        });
        x86_64::instructions::hlt();
    }
}
