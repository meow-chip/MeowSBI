pub struct UART16550 {
    base: usize,
    shift: usize,
    clk: u64,
    baud: u64,
}

mod offsets {
    pub const RBR: usize = 0x0;
    pub const THR: usize = 0x0;

    pub const IER: usize = 0x1;
    pub const FCR: usize = 0x2;
    pub const LCR: usize = 0x3;
    pub const MCR: usize = 0x4;
    pub const LSR: usize = 0x5;

    pub const DLL: usize = 0x0;
    pub const DLH: usize = 0x1;
}

mod masks {
    pub const THRE: u8 = 1 << 5;
    pub const DR: u8 = 1;
}

impl UART16550 {
    pub const fn new(base: usize, shift: usize, clk: u64, baud: u64) -> Self {
        Self { base, shift, clk, baud }
    }

    pub fn init(&self) {
        unsafe {
            core::ptr::write_volatile((self.base + (offsets::LCR << self.shift)) as *mut u8, 0x80); // DLAB

            let latch = self.clk / (16 * self.baud);
            core::ptr::write_volatile(
                (self.base + (offsets::DLL << self.shift)) as *mut u8,
                latch as u8
            );
            core::ptr::write_volatile((self.base + (offsets::DLH << self.shift)) as *mut u8, (latch >> 8) as u8);

            core::ptr::write_volatile((self.base + (offsets::LCR << self.shift)) as *mut u8, 3); // WLEN8 & !DLAB

            core::ptr::write_volatile((self.base + (offsets::MCR << self.shift)) as *mut u8, 0);
            core::ptr::write_volatile((self.base + (offsets::IER << self.shift)) as *mut u8, 0);
            core::ptr::write_volatile((self.base + (offsets::FCR << self.shift)) as *mut u8, 0x7); // FIFO enable + FIFO reset

            // No interrupt for now
        }
    }

    pub fn putchar(&self, c: u8) {
        unsafe {
            core::ptr::write_volatile((self.base + (offsets::THR << self.shift)) as *mut u8, c);

            loop {
                if core::ptr::read_volatile((self.base + (offsets::LSR << self.shift)) as *const u8) & masks::THRE
                    != 0
                {
                    break;
                }
            }
        }
    }

    pub fn getchar(&self) -> u8 {
        unsafe {
            loop {
                if core::ptr::read_volatile((self.base + (offsets::LSR << self.shift)) as *const u8) & masks::DR
                    != 0
                {
                    break;
                }
            }

            core::ptr::read_volatile((self.base + (offsets::RBR << self.shift)) as *const u8)
        }
    }
}
