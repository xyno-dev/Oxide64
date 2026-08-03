use core::sync::atomic::Ordering;
use x86_64::instructions::port::Port;

use crate::interrupts;

pub fn init_pit_channel_0() {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let mut pit_cmd: Port<u8> = Port::new(0x43);
        pit_cmd.write(0b00110110);

        let mut ch0: Port<u8> = Port::new(0x40);
        let divisor = 1193182 / 1000;

        ch0.write((divisor & 0xFF) as u8);
        ch0.write(((divisor >> 8) & 0xFF) as u8);
    });
}

pub fn sleep(millis: u64) {
    let target = interrupts::TICKS.load(Ordering::Relaxed) + millis;
    while interrupts::TICKS.load(Ordering::Relaxed) < target {
        x86_64::instructions::hlt();
    }
}

fn read_rtc(index_address: u8) -> u8 {
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        let mut index_port: Port<u8> = Port::new(0x70);
        let mut data_port: Port<u8> = Port::new(0x71);

        index_port.write(index_address);
        let data = data_port.read();

        ((data >> 4) * 10) + (data & 0x0F)
    })
}

pub fn rtc_secs() -> u8 {
    read_rtc(0x00)
}

pub fn rtc_mins() -> u8 {
    read_rtc(0x02)
}

pub fn rtc_hrs() -> u8 {
    read_rtc(0x04)
}

pub fn secs() -> u64 {
    rtc_secs() as u64 + rtc_mins() as u64 * 60 + rtc_hrs() as u64 * 3600
}
