use super::PlatformOps;
use crate::utils::clint::CLINT;
use crate::utils::uart::UART16550;

pub struct QEMU {
    hartid: usize,
    serial: UART16550,
    clint: CLINT,
}

impl PlatformOps for QEMU {
    fn new(hartid: usize, fdt: fdt::FDT) -> Self {
        use core::convert::TryInto;
        let mut clint_addr = None;
        let mut uart_addr = None;
        for node in fdt.nodes() {
            if node.is_compatible_with("clint0") {
                let addr = node.property("reg").map(|p| p.raw().split_at(core::mem::size_of::<usize>()).0);
                clint_addr = addr.map(|p| usize::from_be_bytes(p.try_into().unwrap()));
            } else if node.is_compatible_with("ns16550a") {
                let addr = node.property("reg").map(|p| p.raw().split_at(core::mem::size_of::<usize>()).0);
                uart_addr = addr.map(|p| usize::from_be_bytes(p.try_into().unwrap()));
            }
        }

        QEMU {
            hartid,
            serial: UART16550::new(uart_addr.unwrap_or(0x10000000)),
            clint: CLINT::new(clint_addr.unwrap_or(0x2000000) as *mut u8, hartid),
        }
    }

    fn early_init(&self, _cold: bool) {
        if self.hartid == 0 {
            self.clint.setup_leader();
            self.serial.init();
        }

        // TODO: barrier here
        self.clint.setup();
    }

    fn set_timer(&self, instant: u64) {
        self.clint.set_timer(instant);
    }

    fn put_char(&self, c: u8) {
        self.serial.putchar(c)
    }

    fn get_char(&self) -> u8 {
        self.serial.getchar()
    }

    fn send_ipi(&self, hartid: usize) {
        self.clint.send_soft(hartid);
    }

    fn clear_ipi(&self) {
        self.clint.clear_soft();
    }
}
