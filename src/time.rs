use core::arch::asm;

use lazy_static::lazy_static;
use x86_64::instructions::{interrupts, port::Port};

lazy_static! {
    static ref TSC_SECS: f64 = 1.0 / cal_tsc() as f64;
}

fn cal_tsc() -> u64 {
    let mut tsc_hertz: u64 = 0;

    interrupts::without_interrupts(|| {
        let start = secs();
        while start + 1 > secs() {
            continue;
        }
        let cal_start = secs();
        let tsc_start = rdtsc();
        while cal_start + 1 > secs() {
            tsc_hertz = rdtsc() - tsc_start;
        }
    });

    tsc_hertz
}

fn rdtsc() -> u64 {
    let edx: u64;
    let eax: u64;

    unsafe {
        asm!(
            "rdtsc", out("edx") edx, out("eax") eax,
            options(readonly, nostack)
        )
    }

    edx << 32 | eax
}

pub fn rtc_secs() -> u8 {
    interrupts::without_interrupts(|| unsafe {
        let mut index_port: Port<u8> = Port::new(0x70);
        let mut data_port: Port<u8> = Port::new(0x71);

        index_port.write(0x00);
        let secs = data_port.read();

        ((secs >> 4) * 10) + (secs & 0x0F)
    })
}

pub fn rtc_mins() -> u8 {
    interrupts::without_interrupts(|| unsafe {
        let mut index_port: Port<u8> = Port::new(0x70);
        let mut data_port: Port<u8> = Port::new(0x71);

        index_port.write(0x04);
        let mins = data_port.read();

        ((mins >> 4) * 10) + (mins & 0x0F)
    })
}

pub fn rtc_hrs() -> u8 {
    interrupts::without_interrupts(|| unsafe {
        let mut index_port: Port<u8> = Port::new(0x70);
        let mut data_port: Port<u8> = Port::new(0x71);

        index_port.write(0x02);
        let hrs = data_port.read();

        ((hrs >> 4) * 10) + (hrs & 0x0F)
    })
}

pub fn secs() -> u64 {
    rtc_secs() as u64 + rtc_mins() as u64 * 60 + rtc_hrs() as u64 * 3600
}

pub fn secs_prec() -> f64 {
    rdtsc() as f64 * *TSC_SECS
}

pub fn sleep(secs: f64) {
    let start = secs_prec();
    while secs_prec() < start + secs {
        continue;
    }
}
