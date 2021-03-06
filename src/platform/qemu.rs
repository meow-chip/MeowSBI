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
        let mut uart_shift: Option<usize> = None;
        let mut uart_offset: Option<usize> = None;
        let mut uart_clk: Option<usize> = None;
        let mut uart_baud: Option<usize> = None;

        crate::serial::early_print("Parsing FDT\n");
        for node in fdt.nodes() {
            if node.is_compatible_with("clint0") {
                let addr = node
                    .property("reg")
                    .map(|p| p.raw().split_at(core::mem::size_of::<usize>()).0);
                clint_addr = addr.map(|p| usize::from_be_bytes(p.try_into().unwrap()));
            } else if node.is_compatible_with("ns16550a") || node.is_compatible_with("na16550"){
                let addr = node
                    .property("reg")
                    .map(|p| p.raw().split_at(core::mem::size_of::<usize>()).0);
                uart_addr = addr.map(|p| usize::from_be_bytes(p.try_into().unwrap()));

                uart_offset = node.property("reg-offset")
                    .and_then(|p| p.as_u32().ok())
                    .map(|r| r as _);

                uart_shift = node.property("reg-shift")
                    .and_then(|p| p.as_u32().ok())
                    .map(|r| r as _);

                uart_clk = node.property("clock-frequency")
                    .and_then(|p| p.as_u32().ok())
                    .map(|r| r as _);

                uart_baud = node.property("current-speed")
                    .and_then(|p| p.as_u32().ok())
                    .map(|r| r as _);
            }
        }

        // crate::serial::early_print("Parsing Finished\n");

        QEMU {
            hartid,
            serial: UART16550::new(
                uart_addr.unwrap_or(0x10000000) + uart_offset.unwrap_or(0),
                uart_shift.unwrap_or(0),
                uart_clk.unwrap_or(11_059_200) as _,
                uart_clk.unwrap_or(115200) as _,
            ),
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
