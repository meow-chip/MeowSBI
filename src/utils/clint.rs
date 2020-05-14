pub struct CLINT {
    base: *mut u8,
    hartid: usize,
}

impl CLINT {
    pub fn new(base: *mut u8, hartid: usize) -> CLINT {
        CLINT { base, hartid }
    }

    pub fn setup_leader(&self) {
        // Setup mtime
        unsafe {
            core::ptr::write_volatile(self.base.offset(0xbff8) as *mut u64, 0);
        }
    }

    pub fn setup(&self) {
        // Writes timecmp to no timer
        unsafe {
            core::ptr::write_volatile(
                (self.base.offset(0x4000) as *mut u64).offset(self.hartid as isize),
                core::u64::MAX >> 4, // Fix QEMU timer loop
            );
        }

        // Clears all software interrupts for current HART
        unsafe {
            core::ptr::write_volatile((self.base as *mut u32).offset(self.hartid as isize), 0);
        }
    }

    pub fn set_timer(&self, instant: u64) {
        unsafe {
            core::ptr::write_volatile(
                (self.base.offset(0x4000) as *mut u64).offset(self.hartid as isize),
                instant,
            );
        }
    }
}
