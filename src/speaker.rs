use x86_64::instructions::port::Port;

use crate::time;

const PIT_FREQ: u32 = 1193182;
const SPEAKER_ENABLE: u8 = 0b00000011;

pub fn init_pit_channel_2() {
    unsafe {
        let mut pit_cmd: Port<u8> = Port::new(0x43);
        pit_cmd.write(0b10110110);
    }
}

pub fn beep(frequency: f32, duration: u64) {
    unsafe {
        let mut pc_speaker: Port<u8> = Port::new(0x61);
        let pc_speaker_data = pc_speaker.read() | SPEAKER_ENABLE;
        pc_speaker.write(pc_speaker_data);

        let divisor = (PIT_FREQ as f32 / frequency) as u16;
        let mut ch2: Port<u8> = Port::new(0x42);
        ch2.write((divisor & 0xFF) as u8);
        ch2.write(((divisor >> 8) & 0xFF) as u8);

        time::sleep(duration);

        let mut pc_speaker: Port<u8> = Port::new(0x61);
        let pc_speaker_data = pc_speaker.read() & !SPEAKER_ENABLE;
        pc_speaker.write(pc_speaker_data);
    }
}
