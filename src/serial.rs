use crate::platform::PlatformOps;
use core::sync::atomic::*;

mod locking {
    use core::sync::atomic::*;
    #[link_section = ".sdata"]
    pub static READ: AtomicBool = AtomicBool::new(false);
    #[link_section = ".sdata"]
    pub static WRITE: AtomicBool = AtomicBool::new(false);
    #[link_section = ".sdata"]
    pub static PRINT: AtomicBool = AtomicBool::new(false);
}

pub fn putc(c: u8) {
    /*
    while locking::WRITE.compare_and_swap(false, true, Ordering::Acquire) {
        spin_loop_hint();
    }
    */

    crate::mem::local_data().platform().put_char(c);

    // locking::WRITE.store(false, Ordering::Release);
}

pub fn getc() -> u8 {
    while locking::READ.compare_and_swap(false, true, Ordering::Acquire) {
        spin_loop_hint();
    }

    let ret = crate::mem::local_data().platform().get_char();

    locking::READ.store(false, Ordering::Release);

    ret
}

pub fn print(s: &str) {
    for c in s.as_bytes() {
        putc(*c);
    }
}

use core::fmt::Write;
struct MeowSBIStdout;

impl Write for MeowSBIStdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s);
        // early_print(s);
        Ok(())
    }
}

pub fn fprint(args: core::fmt::Arguments) -> core::fmt::Result {
    /*
    while locking::PRINT.compare_and_swap(false, true, Ordering::Acquire) {
        spin_loop_hint();
    }
    */

    let result = MeowSBIStdout.write_fmt(args);

    // locking::PRINT.store(false, Ordering::Release);

    result
}

#[macro_export]
macro_rules! mprint {
    ($($arg:tt)*) => ({
        $crate::serial::fprint(format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! mprintln {
    () => ($crate::mprint!("\n"));
    ($($arg:tt)*) => ($crate::mprint!("{}\n", format_args!($($arg)*)));
}

// Early print
const EARLY_SERIAL: crate::utils::uart::UART16550
    = crate::utils::uart::UART16550::new(0x10001000, 2, 11_059_200, 115200);

pub fn early_print(s: &str) {
    for c in s.bytes() {
        unsafe {
            EARLY_SERIAL.putchar(c);
        }
    }
}

pub fn early_print_setup() {
    EARLY_SERIAL.init();
}
