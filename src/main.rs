#![allow(incomplete_features)]
#![feature(
    const_generics,
    link_args,
    naked_functions,
    llvm_asm,
    global_asm,
    const_transmute
)]
#![cfg_attr(not(test), no_std)]
#![no_main]

#![allow(unused_attributes)]
#![link_args = "-Tsrc/ld/qemu.ld"]

use riscv;

mod boot;
mod lang_items;
mod mem;
mod platform;
mod sbi;
mod serial;
mod trap;
mod utils;

use platform::PlatformOps;

const HART_CNT: usize = 1;
const HART_STORE_SIZE: usize = 4096;
#[macro_export]
macro_rules! HART_STORE_SHIFT_STR {
    () => {
        "12"
    };
}

type PLATFORM = platform::qemu::QEMU;

use core::sync::atomic::*;
static FDT_RELOCATE_TOKEN: AtomicBool = AtomicBool::new(false);
static FDT_RELOCATED_ADDR: AtomicUsize = AtomicUsize::new(0);

/**
 * MeowSBI entry point
 * Primary boot entry is at boot::entry
 */

#[no_mangle]
extern "C" fn boot(hartid: usize, fdt_addr: *const u8) -> ! {
    let fdt_addr = if hartid == 0 {
        let fdt_addr = relocate_fdt(fdt_addr);
        FDT_RELOCATED_ADDR.store(fdt_addr as usize, Ordering::Relaxed);
        FDT_RELOCATE_TOKEN.store(true, Ordering::Release);
        fdt_addr
    } else {
        while FDT_RELOCATE_TOKEN.load(Ordering::Acquire) {
            spin_loop_hint();
        }

        FDT_RELOCATED_ADDR.load(Ordering::Relaxed) as *const u8
    };

    let fdt = unsafe { fdt::FDT::from_raw(fdt_addr) }.unwrap();

    // Initialize platform
    unsafe {
        core::ptr::write(mem::data(hartid).platform.as_mut_ptr(), PLATFORM::new(hartid, fdt));
    }

    // Print MOTD
    mprint!(include_str!("./motd.txt")).unwrap();

    // Setup mtvec
    trap::setup();

    next_boot(hartid, fdt_addr);
}

// FW_JUMP mode
fn next_boot(hartid: usize, fdt_addr: *const u8) -> ! {
    unsafe {
        riscv::register::mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
        riscv::register::mepc::write(0x80200000);
        trap::next_ret(hartid, fdt_addr);
    }
}

static mut FDT_STORAGE: [u8; 16384] = [0; 16384];
static FDT_RELOCATE_TICKET: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0); 

fn relocate_fdt(original: *const u8) -> *const u8 {
    use core::sync::atomic::Ordering;
    let ticket = FDT_RELOCATE_TICKET.fetch_add(1, Ordering::SeqCst);

    let ret = unsafe { &mut FDT_STORAGE[0] as _ };

    if ticket != 0 {
        // Waits for ticket to become usize::max
        while FDT_RELOCATE_TICKET.load(Ordering::Acquire) != core::usize::MAX {
            core::sync::atomic::spin_loop_hint();
        }
    } else {
        let parsed = unsafe { fdt::FDT::from_raw(original) }.unwrap();
        let size = parsed.total_size();
        unsafe {
            core::ptr::copy_nonoverlapping(original, ret, size as usize);
        }

        FDT_RELOCATE_TICKET.store(core::usize::MAX, Ordering::Release)
    }

    ret
}
