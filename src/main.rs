#![feature(const_generics, link_args, naked_functions, asm, global_asm, const_transmute)]

#![cfg_attr(not(test), no_std)]
#![no_main]

#![link_args = "-Tsrc/ld/qemu.ld"]

use riscv;

mod platform;
mod utils;
mod trap;
mod lang_items;
mod sbi;
mod serial;
mod boot;
mod mem;

use platform::PlatformOps;

const HART_CNT: usize = 1;
const HART_STORE_SIZE: usize = 4096;
#[macro_export]
macro_rules! HART_STORE_SHIFT_STR {
    () => { "12" }
}

type PLATFORM = platform::qemu::QEMU;

/**
 * MeowSBI entry point
 * Primary boot entry is at boot::entry
 */

pub fn boot() -> ! {
    let hartid = riscv::register::mhartid::read();
    PLATFORM::on(hartid).early_init(true);

    // Print MOTD
    mprint!(include_str!("./motd.txt")).unwrap();

    // Setup mtvec
    trap::setup();

    next_boot();
}

// FW_JUMP mode
fn next_boot() -> ! {
    unsafe {
        riscv::register::mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
        riscv::register::mepc::write(0x80200000);
        trap::next_ret();
    }
}
