pub struct CLINT<const BASE: usize> {
    hartid: usize,
}

impl<const BASE: usize> CLINT<BASE> {
    pub fn with(hartid: usize) -> CLINT<BASE> {
        CLINT { hartid }
    }

    pub fn setup_leader() {
        // Setup mtime
        unsafe {
            core::ptr::write_volatile((BASE + 0xbff8) as *mut u64, 0);
        }
    }

    pub fn setup(&self) {
        // Writes timecmp to no timer
        unsafe {
            core::ptr::write_volatile(
                ((BASE + 0x4000) as *mut u64).offset(self.hartid as isize),
                core::u64::MAX >> 4, // Fix QEMU timer loop
            );
        }

        // Clears all software interrupts for current HART
        unsafe {
            core::ptr::write_volatile((BASE as *mut u32).offset(self.hartid as isize), 0);
        }
    }

    pub fn set_timer(&self, instant: u64) {
        unsafe {
            core::ptr::write_volatile(
                ((BASE + 0x4000) as *mut u64).offset(self.hartid as isize),
                instant,
            );
        }
    }
}
