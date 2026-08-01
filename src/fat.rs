use core::mem::transmute;
use x86_64::instructions::{self, port::Port};

use crate::println;

#[derive(Debug)]
#[repr(C, packed)]
struct Bpb {
    jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    root_dir_entries: u16,
    total_sectors: u16,
    media_desc_type: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    sides: u16,
    hidden_sectors: u32,
    large_sector_count: u32
}

unsafe fn read_sectors(lbalo: u8, lbamid: u8, lbahi: u8, sectors: u8) -> [u16; 256] {
    unsafe {
        let io_base = 0x1F0;
        let mut data_register = Port::<u16>::new(io_base);
        let mut command_register = Port::<u8>::new(io_base + 7);
        let mut sector_count = Port::<u8>::new(io_base + 2);

        let mut lba_low = Port::<u8>::new(io_base + 3);
        let mut lba_mid = Port::<u8>::new(io_base + 4);
        let mut lba_high = Port::<u8>::new(io_base + 5);

        sector_count.write(sectors);
        lba_low.write(lbalo);
        lba_mid.write(lbamid);
        lba_high.write(lbahi);
        
        Port::<u8>::new(io_base + 6).write(0xE0);

        command_register.write(0x20);
        
        let mut response = command_register.read();

        while ((response >> 7) & 0b01) == 1 || ((response >> 3) & 1) != 1 {
            response = command_register.read();
            println!("{response:b}");
        }

        let mut data = [0 as u16; 256];

        for i in 0..256 {
            data[i] = data_register.read()
        }

        data
    }
}

fn read_bpb() {
    unsafe {
        let sector_zero = read_sectors(0, 0, 0, 1);
        let bpb_data: &[u16] = &sector_zero[..18];

        println!("RAW 16 PACKETS FROM SECTOR 0:\n{:?}\n", bpb_data);

        let bpb: Bpb = transmute::<[u16; 18], Bpb>(bpb_data.try_into().unwrap());
        println!("BPB STRUCT:\n{:?}\n", bpb);
        println!("FAT16 OEM IDENTIFIER: {}\n", str::from_utf8_unchecked(&bpb.oem));
    }
}

pub fn init() {
    instructions::interrupts::without_interrupts(|| {
        unsafe {
            let io_base = 0x1F0;
            let mut data_register = Port::<u16>::new(io_base);
            let mut command_register = Port::<u8>::new(io_base + 7);
            let mut sector_count = Port::<u8>::new(io_base + 2);

            let mut lba_low = Port::<u8>::new(io_base + 3);
            let mut lba_mid = Port::<u8>::new(io_base + 4);
            let mut lba_high = Port::<u8>::new(io_base + 5);

            Port::<u8>::new(0x3F6).write(0x02); 

            sector_count.write(1);
            lba_low.write(0);
            lba_mid.write(0);
            lba_high.write(0);

            Port::<u8>::new(io_base + 6).write(0xE0);

            command_register.write(0xEC);
            let mut response = command_register.read();
            if response == 0 {
                panic!("\
                    No ATA storage detected!\n\
                    Is there a hard drive plugged in?\
                ")
            }

            if Port::<u16>::new(io_base + 4).read() != 0 ||
                Port::<u16>::new(io_base + 5).read() != 0
            {
                println!("\
                    Disk is not a base ATA device!\n\
                    Are you using a hard drive?\n\
                    Continuing anyway...
                ")
            }

            while ((response >> 7) & 0b01) == 1 || ((response >> 3) & 1) != 1 {
                response = command_register.read();
                println!("{response:b}");
            }

            for _ in 0..256 {
                data_register.read();
            }
            
            read_bpb();
        }
    });
}
