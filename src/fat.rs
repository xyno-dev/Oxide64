use core::mem::transmute;
use heapless::Vec;
use lazy_static::lazy_static;
use x86_64::instructions::{self, port::Port};

use crate::println;

lazy_static! {
    static ref BPB: Bpb = read_bpb();
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Bpb {
    pub jmp: [u8; 3],
    pub oem: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    pub root_dir_entries: u16,
    pub total_sectors: u16,
    pub media_desc_type: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub sides: u16,
    pub hidden_sectors: u32,
    pub large_sector_count: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Ebpb {
    pub drive: u8,
    pub nt_flags: u8,
    pub signature: u8,
    pub volume_serial: u32,
    pub volume_label: [u8; 11],
    pub sys_id: [u8; 8],
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Directory {
    pub name: [u8; 8],
    pub extension: [u8; 3],
    pub attributes: u8,
    pub reserved: u8,
    pub time_taken: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_accessed: u16,
    pub cluster_high: u16,
    pub last_modified_time: u16,
    pub last_modified_date: u16,
    pub cluster_low: u16,
    pub size: u32,
}

impl Directory {
    pub fn new(name: &str, extension: &str, size: u32) -> Self {
        Self {
            name: name.as_bytes().try_into().unwrap(),
            extension: extension.as_bytes().try_into().unwrap(),
            attributes: 0,
            reserved: 0,
            time_taken: 0,
            creation_time: 1,
            creation_date: 1,
            last_accessed: 1,
            cluster_high: 0,
            last_modified_time: 1,
            last_modified_date: 1,
            cluster_low: 0,
            size,
        }
    }
}

pub struct FileStream {
    directory: Directory,
    cluster: u16,
    pointer: u16,
}

impl FileStream {
    pub fn new(directory: Directory) -> Self {
        let first_cluster = directory.cluster_low;
        Self {
            directory,
            cluster: first_cluster,
            pointer: 0,
        }
    }

    fn next_cluster(&mut self) -> Option<u16> {
        let first_fat_sector = BPB.reserved_sectors;
        let fat_offset = self.cluster / 512;
        let sector_address = (first_fat_sector + fat_offset) as u32;
        let byte_offset = self.cluster - fat_offset;

        if fat_offset > BPB.sectors_per_fat {
            None
        } else {
            unsafe {
                let sector = read_sector(sector_address);

                self.cluster = u16::from_le_bytes([
                    sector[byte_offset as usize],
                    sector[byte_offset as usize + 1],
                ]);

                Some(self.cluster)
            }
        }
    }
}

unsafe fn wait_until_data_ready() {
    unsafe {
        let io_base = 0x1F0;
        let mut command_register = Port::<u8>::new(io_base + 7);

        let mut response = command_register.read();

        while ((response >> 7) & 0b01) == 1 || ((response >> 3) & 1) != 1 {
            response = command_register.read();
        }
    }
}

unsafe fn wait_until_bsy_clear() {
    unsafe {
        let io_base = 0x1F0;
        let mut command_register = Port::<u8>::new(io_base + 7);

        let mut response = command_register.read();

        while ((response >> 7) & 0b01) == 1 {
            response = command_register.read();
            println!("{response}");
        }
    }
}

unsafe fn read_sector(lba_addr: u32) -> [u8; 512] {
    unsafe {
        let io_base = 0x1F0;
        let mut data_register = Port::<u16>::new(io_base);
        let mut command_register = Port::<u8>::new(io_base + 7);
        let mut sector_count = Port::<u8>::new(io_base + 2);

        let mut lba_low = Port::<u8>::new(io_base + 3);
        let mut lba_mid = Port::<u8>::new(io_base + 4);
        let mut lba_high = Port::<u8>::new(io_base + 5);

        sector_count.write(1);
        lba_low.write((lba_addr & 0xFF) as u8);
        lba_mid.write((lba_addr >> 8 & 0xFF) as u8);
        lba_high.write((lba_addr >> 16 & 0xFF) as u8);

        Port::<u8>::new(io_base + 6).write(0xE0);

        command_register.write(0x20);

        wait_until_data_ready();

        let mut data = [0 as u16; 256];

        for i in 0..256 {
            data[i] = data_register.read()
        }

        wait_until_bsy_clear();

        transmute(data)
    }
}

unsafe fn write_sector(lba_addr: u32, sector: [u8; 512]) {
    unsafe {
        let io_base = 0x1F0;
        let mut data_register = Port::<u16>::new(io_base);
        let mut command_register = Port::<u8>::new(io_base + 7);
        let mut sector_count = Port::<u8>::new(io_base + 2);

        let mut lba_low = Port::<u8>::new(io_base + 3);
        let mut lba_mid = Port::<u8>::new(io_base + 4);
        let mut lba_high = Port::<u8>::new(io_base + 5);

        sector_count.write(1);
        lba_low.write((lba_addr & 0xFF) as u8);
        lba_mid.write((lba_addr >> 8 & 0xFF) as u8);
        lba_high.write((lba_addr >> 16 & 0xFF) as u8);

        Port::<u8>::new(io_base + 6).write(0xE0);

        command_register.write(0x30);

        wait_until_data_ready();

        let data: [u16; 256] = transmute(sector);

        for i in 0..256 {
            data_register.write(data[i])
        }

        wait_until_bsy_clear();
    }
}

fn calc_first_root_dir_sector(bpb: &Bpb) -> u16 {
    let root_dir_sectors =
        (bpb.root_dir_entries * 32) + (bpb.bytes_per_sector - 1) / bpb.bytes_per_sector;
    let first_data_sector =
        bpb.reserved_sectors + (bpb.fats as u16 * bpb.sectors_per_fat) + root_dir_sectors;

    first_data_sector - root_dir_sectors
}

pub fn read_bpb() -> Bpb {
    unsafe {
        let sector_zero = read_sector(0);
        let bpb_data: &[u8] = &sector_zero[..36];

        let bpb: Bpb = transmute::<[u8; 36], Bpb>(bpb_data.try_into().unwrap());

        bpb
    }
}

pub fn read_ebpb() -> Ebpb {
    unsafe {
        let sector_zero = read_sector(0);
        let ebpb_data: &[u8] = &sector_zero[36..62];

        let ebpb: Ebpb = transmute::<[u8; 26], Ebpb>(ebpb_data.try_into().unwrap());

        ebpb
    }
}

pub fn list_directories(bpb: &Bpb) -> Vec<Directory, 512> {
    let first_root_dir_sector_number = calc_first_root_dir_sector(&bpb) as u32;
    let root_dir_sectors = ((bpb.root_dir_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1))
        / bpb.bytes_per_sector as u32;

    let mut entries: Vec<Directory, 512> = Vec::new();
    let mut zero_count = 0;

    unsafe {
        for i in 0..root_dir_sectors {
            let sector = read_sector(first_root_dir_sector_number + i);
            for (i, byte) in sector.iter().enumerate() {
                if *byte == 0 {
                    zero_count += 1;
                } else {
                    if zero_count > 4 {
                        entries
                            .push(transmute::<[u8; 32], Directory>(
                                sector[i..(i + 32)].try_into().unwrap(),
                            ))
                            .unwrap();
                    }
                    zero_count = 0;
                }
            }
        }
    }

    entries
}

pub fn read_file(entry: &Directory, bpb: &Bpb) -> Vec<u8, 1024> {
    let sectors_per_cluster = bpb.sectors_per_cluster as u16;
    let root_dir_sectors = ((bpb.root_dir_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1))
        / bpb.bytes_per_sector as u32;
    let data_start_sector = bpb.reserved_sectors as u32
        + (bpb.fats as u32 * bpb.sectors_per_fat as u32)
        + root_dir_sectors as u32;

    let entry_size = entry.size;
    let cluster_low = entry.cluster_low;
    let first_sector = data_start_sector + (cluster_low as u32 - 2) * sectors_per_cluster as u32;

    let first_fat_sector = bpb.reserved_sectors;
    let fat_offset = cluster_low * 2;
    let fat_sector = first_fat_sector + (fat_offset / 512);
    let ent_offset = (fat_offset % 512) as usize;

    unsafe {
        let fat_table = read_sector(fat_sector as u32);
        let table_value: u16 =
            u16::from_le_bytes([fat_table[ent_offset], fat_table[ent_offset + 1]]);
        println!("{table_value:X}");

        let data_sector = read_sector(first_sector as u32);
        Vec::from_slice(&data_sector[0..entry_size as usize]).unwrap()
    }
}

pub fn create_file(mut entry: Directory, bpb: &Bpb) {
    let first_root_dir_sector_number = calc_first_root_dir_sector(&bpb) as u32;
    let root_dir_sectors = ((bpb.root_dir_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1))
        / bpb.bytes_per_sector as u32;

    let mut clusters_needed = entry
        .size
        .div_ceil(bpb.bytes_per_sector as u32 * bpb.sectors_per_cluster as u32);

    unsafe {
        let first_fat_sector = bpb.reserved_sectors;
        let sectors_per_fat = bpb.sectors_per_fat as usize;

        let mut empty_clusters: Vec<u16, 512> = Vec::new();
        let mut sector_addresses: Vec<u32, 512> = Vec::new();

        'sector_loop: for offset in 0..sectors_per_fat {
            let sector: [u16; 256] =
                transmute(read_sector(first_fat_sector as u32 + offset as u32));
            for (i, word) in sector[2..].iter().enumerate() {
                if clusters_needed <= 0 {
                    break 'sector_loop;
                }
                if *word == 0 {
                    empty_clusters.push(i as u16 + 2).unwrap();
                    sector_addresses
                        .push(first_fat_sector as u32 + offset as u32)
                        .unwrap();
                    clusters_needed -= 1;
                }
            }
        }

        for (i, cluster_address) in empty_clusters.clone().iter().enumerate() {
            let sector_address = sector_addresses[i];
            let mut sector: [u16; 256] = transmute(read_sector(sector_address));

            if i == 0 {
                entry.cluster_low = *cluster_address;
            } else if i == empty_clusters.len() - 1 {
                sector[*cluster_address as usize] = 0xFFFF;
                sector[empty_clusters[i - 1] as usize] = *cluster_address;
            } else {
                sector[empty_clusters[i - 1] as usize] = *cluster_address;
            }

            write_sector(sector_address, transmute(sector));
            write_sector(sector_address + sectors_per_fat as u32, transmute(sector));
        }

        println!("{entry:?}");

        let entry_bytes: [u8; 32] = transmute(entry);

        'sector_loop: for i in 0..root_dir_sectors {
            let sector_address = first_root_dir_sector_number + i;
            let mut sector = read_sector(sector_address);

            for i in (0..512).step_by(32) {
                if sector[i] == 0 {
                    sector[i..(i + 32)].copy_from_slice(&entry_bytes);
                    write_sector(sector_address, sector);
                    println!("Found free slot: {i}");
                    break 'sector_loop;
                }
            }
        }
    }
}

pub fn init() {
    instructions::interrupts::without_interrupts(|| unsafe {
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
        let response = command_register.read();
        if response == 0 {
            panic!(
                "\
                    No ATA storage detected!\n\
                    Is there a hard drive plugged in?\
                "
            )
        }

        if Port::<u16>::new(io_base + 4).read() != 0 || Port::<u16>::new(io_base + 5).read() != 0 {
            println!(
                "\
                    Disk is not a base ATA device!\n\
                    Are you using a hard drive?\n\
                    Continuing anyway...
                "
            )
        }

        wait_until_data_ready();

        for _ in 0..256 {
            data_register.read();
        }

        let bpb = read_bpb();
        read_ebpb();
        println!("SECTORS PER CLUSTER: {}", &bpb.sectors_per_cluster);

        for entry in list_directories(&bpb) {
            let contents = read_file(&entry, &bpb);
            println!(
                "CONTENTS READ!\n{}\nEOF",
                str::from_utf8_unchecked(&contents)
            );
        }

        let new_file_entry = Directory::new("tungtung", "txt", 8192);

        create_file(new_file_entry, &bpb);
    });
}
