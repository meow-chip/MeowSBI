use crate::platform::PlatformOps;
use core::sync::atomic::*;

mod locking {
    use core::sync::atomic::*;
    #[link_section = ".sdata"]
    pub static READ: AtomicBool = AtomicBool::new(false);
    #[link_section = ".sdata"]
    pub static WRITE: AtomicBool = AtomicBool::new(false);
}

pub fn putc(c: u8) {
    while locking::WRITE.swap(true, Ordering::Acquire) {
        spin_loop_hint();
    }

    crate::PLATFORM::local().put_char(c);

    locking::WRITE.store(false, Ordering::Release);
}

pub fn getc() -> u8 {
    while locking::READ.swap(true, Ordering::Acquire) {
        spin_loop_hint();
    }

    let ret = crate::PLATFORM::local().get_char();

    locking::WRITE.store(false, Ordering::Release);

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
        Ok(())
    }
}

pub fn fprint(args: core::fmt::Arguments) -> core::fmt::Result {
    MeowSBIStdout.write_fmt(args)
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
