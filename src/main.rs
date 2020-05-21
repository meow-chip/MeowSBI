#![allow(incomplete_features)]
#![feature(
    const_generics,
    link_args,
    naked_functions,
    llvm_asm,
    global_asm,
    const_transmute
)]
#![feature(const_in_array_repeat_expressions)]
#![cfg_attr(not(test), no_std)]
#![no_main]
#![allow(unused_attributes)]
#![allow(non_camel_case_types)]
#![link_args = "-Tsrc/provided/linker.ld"]

use riscv;

mod boot;
mod ipi;
mod lang_items;
mod mem;
mod platform;
mod sbi;
mod serial;
mod trap;
mod utils;

use platform::PlatformOps;

const HART_CNT: usize = 2;
const HART_STORE_SIZE: usize = 1 << 16;
#[macro_export]
macro_rules! HART_STORE_SHIFT_STR {
    () => {
        "16"
    };
}

type PLATFORM = platform::meowv64::MeowV64;

use core::sync::atomic::*;
static mut FDT_RELOCATED_ADDR: *mut u8 = 0 as *mut u8;
static WARM_BOOT_FIRE: AtomicBool = AtomicBool::new(false);

extern "C" {
    fn _fw_start();
    fn _fw_end();
}

/**
 * MeowSBI entry point
 * Primary boot entry is at boot::entry
 */

#[no_mangle]
extern "C" fn boot(hartid: usize, fdt_addr: *const u8) -> ! {
    if hartid != 0 {
        warm_boot(hartid)
    }

    // Early boot routine

    // Relocate fdt
    let embedded_fdt = include_bytes!("provided/dt.fdt");
    let fdt_addr = if fdt_addr == core::ptr::null() { embedded_fdt as *const u8 } else { fdt_addr };
    let fdt_addr = relocate_fdt(fdt_addr);
    unsafe { FDT_RELOCATED_ADDR = fdt_addr };

    let fdt = unsafe { fdt::FDT::from_raw(fdt_addr) }.unwrap();

    // Initialize platform
    unsafe {
        core::ptr::write(
            mem::data(hartid).platform.as_mut_ptr(),
            PLATFORM::new(hartid, fdt),
        );
    }

    mprint!(include_str!("./motd.txt")).unwrap();

    crate::mprintln!("FDT relocated to 0x{:016X}", fdt_addr as usize).unwrap();

    // Setup pmp
    setup_pmp();

    // Setup mtvec
    trap::setup();

    // Fixup fdt
    fixup_fdt(fdt_addr);
    crate::mprintln!("Hart {} cold boot... arg1: 0x{:016x}", hartid, fdt_addr as usize).unwrap();
    WARM_BOOT_FIRE.store(true, Ordering::Release);

    next_boot(hartid, fdt_addr);
}

fn warm_boot(hartid: usize) -> ! {
    while WARM_BOOT_FIRE.load(Ordering::Acquire) == false {
        spin_loop_hint();
    }

    let fdt_addr = unsafe { FDT_RELOCATED_ADDR };

    let fdt = unsafe { fdt::FDT::from_raw(fdt_addr) }.unwrap();
    unsafe {
        core::ptr::write(
            mem::data(hartid).platform.as_mut_ptr(),
            PLATFORM::new(hartid, fdt),
        );
    }

    setup_pmp();
    trap::setup();

    crate::mprintln!("Hart {} warm boot... arg1: 0x{:016x}", hartid, fdt_addr as usize).unwrap();
    next_boot(hartid, fdt_addr);
}

// FW_JUMP mode
fn next_boot(hartid: usize, fdt_addr: *const u8) -> ! {
    unsafe {
        riscv::register::stvec::write(0x80200000, riscv::register::stvec::TrapMode::Direct);
        riscv::register::sscratch::write(0);
        riscv::register::sie::clear_sext();
        riscv::register::sie::clear_ssoft();
        riscv::register::sie::clear_stimer();
        riscv::register::satp::write(0);

        riscv::register::mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
        riscv::register::mepc::write(0x80200000);
        trap::next_ret(hartid, fdt_addr);
    }
}

// static mut FDT_STORAGE: [u8; 16384] = [0; 16384];
const FDT_STORAGE_START: *mut u8 = 0x82200000usize as _;

fn relocate_fdt(original: *const u8) -> *mut u8 {
    let parsed = unsafe { fdt::FDT::from_raw(original) }.unwrap();
    let size = parsed.total_size();
    unsafe {
        core::ptr::copy_nonoverlapping(original, FDT_STORAGE_START, size as usize);
    }

    FDT_STORAGE_START
}

fn fixup_fdt(fdt: *mut u8) {
    // Load rsvmap offset
    let rvsmap_offset = u32::from_be_bytes(unsafe { *(fdt.offset(16) as *const [u8; 4]) });
    let struct_offset = u32::from_be_bytes(unsafe { *(fdt.offset(8) as *const [u8; 4]) });
    let rvsmap_raw = unsafe { fdt.offset(rvsmap_offset as isize) };

    // Find first pair of zeros
    for i in 0.. { // FIXME: limit to fdt size
        let addr = unsafe { u32::from_be_bytes(*(rvsmap_raw.offset(i*8) as *mut [u8;4])) };
        let len = unsafe { u32::from_be_bytes(*(rvsmap_raw.offset(i*8+4) as *mut [u8;4])) };

        if addr == 0 && len == 0 {
            // Self is empty
            if (i as u32 + 1) * 8 + rvsmap_offset == struct_offset {
                crate::mprintln!("Insufficient space for additional memory reservation entry. Struct offset at {}, rvs offset at {}", struct_offset, rvsmap_offset).unwrap();
                panic!();
            }

            // Clear next entry
            unsafe { *(rvsmap_raw.offset(i*8 + 8) as *mut u64) = 0 };

            // Fill in current entry
            unsafe {
                *(rvsmap_raw.offset(i*8) as *mut [u8;4]) = u32::to_be_bytes(0x80000000);
                *(rvsmap_raw.offset(i*8+4) as *mut [u8;4]) = u32::to_be_bytes(0x200000);
            }

            break;
        } else {
            crate::mprintln!("Get reservation entry: 0x{:08X}, len: 0x{:08X}", addr, len).unwrap();
        }
    }
}

fn setup_pmp() {
    // Setup PMP for firmware itself
    let fw_start = 0x8000_0000;
    let fw_size = (_fw_end as usize - _fw_start as usize).next_power_of_two();

    let addr_encoded = (fw_start >> 2) | ((fw_size >> 3) - 1);

    crate::mprintln!("FW size: {:016X}", fw_size).unwrap();
    crate::mprintln!("PMP encoded: {:016X}", addr_encoded).unwrap();

    riscv::register::pmpaddr0::write(addr_encoded);
    riscv::register::pmpaddr1::write((1 << (56-3)) - 1); // Maximum range
    riscv::register::pmpcfg0::write(
        (
            (3 << 3) | 0 // NAPOT + not locking
        ) | (
            ((3 << 3) | 7) << 8
        )
    );
}
