pub struct UART16550<const BASE: usize>;

mod offsets {
    pub const MULTIPLY: usize = 1;
    pub const RBR: usize = 0x0;
    pub const THR: usize = 0x0;

    pub const IER: usize = 0x1 * MULTIPLY;
    pub const FCR: usize = 0x2 * MULTIPLY;
    pub const LCR: usize = 0x3 * MULTIPLY;
    pub const MCR: usize = 0x3 * MULTIPLY;
    pub const LSR: usize = 0x5 * MULTIPLY;

    pub const DLL: usize = 0x0;
    pub const DLH: usize = 0x1 * MULTIPLY;
}

mod masks {
    pub const THRE: u8 = 1 << 5;
    pub const DR: u8 = 1;
}

impl<const BASE: usize> UART16550<BASE> {
    pub fn init() {
        unsafe {
            core::ptr::write_volatile((BASE + offsets::FCR) as *mut u8, 0x7); // FIFO enable + FIFO reset

            core::ptr::write_volatile((BASE + offsets::LCR) as *mut u8, 0x80); // DLAB
            core::ptr::write_volatile((BASE + offsets::DLL) as *mut u8, (115200u64 / 9600u64) as u8);
            core::ptr::write_volatile((BASE + offsets::DLH) as *mut u8, 0);

            core::ptr::write_volatile((BASE + offsets::LCR) as *mut u8, 0x03 & !0x80u8); // WLEN8 & !DLAB
            core::ptr::write_volatile((BASE + offsets::MCR) as *mut u8, 0); // WLEN8 & !DLAB
            core::ptr::write_volatile((BASE + offsets::IER) as *mut u8, 0); // No interrupt for now
        }
    }

    pub fn putchar(c: u8) {
        unsafe {
            core::ptr::write_volatile((BASE + offsets::THR) as *mut u8, c);

            loop {
                if core::ptr::read_volatile((BASE + offsets::LSR) as *const u8) & masks::THRE != 0 {
                    break;
                }
            }
        }
    }

    pub fn getchar() -> u8 {
        unsafe {
            loop {
                if core::ptr::read_volatile((BASE + offsets::LSR) as *const u8) & masks::DR != 0 {
                    break;
                }
            }

            core::ptr::read_volatile((BASE + offsets::THR) as *const u8)

        }
    }
}
