use super::PlatformOps;
use crate::utils::clint::CLINT;
use crate::utils::uart::UART16550;

pub struct QEMU {
    hartid: usize,
}

type QEMU_CLINT = CLINT<0x2000000>;
type QEMU_UART = UART16550<0x10000000>;

impl PlatformOps for QEMU {
    fn on(hartid: usize) -> Self {
        QEMU { hartid }
    }

    fn early_init(&self, _cold: bool) {
        if self.hartid == 0 {
            QEMU_CLINT::setup_leader();
        }

        if self.hartid == 0 {
            QEMU_UART::init();
        }

        QEMU_CLINT::with(self.hartid).setup();
    }

    fn set_timer(&self, instant: u64) {
        QEMU_CLINT::with(self.hartid).set_timer(instant);
    }

    fn put_char(&self, c: u8) {
        QEMU_UART::putchar(c as u8);
    }

    fn get_char(&self) -> u8 {
        QEMU_UART::getchar()
    }
}
