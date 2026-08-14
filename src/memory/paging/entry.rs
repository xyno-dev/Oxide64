use crate::memory::Frame;

pub enum EntryFlags {
    Present = 1 << 0,
    Writable = 1 << 1,
    UserAccessible = 1 << 2,
    WriteThrough = 1 << 3,
    NoCache = 1 << 4,
    Accessed = 1 << 5,
    Dirty = 1 << 6,
    HugePage = 1 << 7,
    Global = 1 << 8,
    NoExecute = 1 << 63,
}

pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn contains_flag(&self, flag: u64) -> bool {
        self.0 & flag == flag
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.contains_flag(EntryFlags::Present as u64) {
            Some(Frame::containing_address(
                self.0 as usize & 0x000fffff_fffff000,
            ))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: u64) {
        assert!(frame.start_address() & !0x000fffff_fffff000 == 0);
        self.0 = (frame.start_address() as u64) | flags;
    }
}
