use core::arch::asm;

pub fn debug_byte(b: u8) {
    unsafe {
        asm!("out dx, al", in("dx") 0x402u16, in("al") b);
    }
}

pub fn debug_str(s: &str) {
    for b in s.bytes() {
        debug_byte(b);
    }
}
