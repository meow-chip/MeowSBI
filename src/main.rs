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
#![link_args = "-Tsrc/ld/qemu.ld"]

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
static FDT_FIXMP_TOKEN: AtomicBool = AtomicBool::new(false);
static mut FDT_RELOCATED_ADDR: *mut u8 = 0 as *mut u8;

/**
 * MeowSBI entry point
 * Primary boot entry is at boot::entry
 */

#[no_mangle]
extern "C" fn boot(hartid: usize, fdt_addr: *const u8) -> ! {
    // Early boot routine
    let fdt_addr = if hartid == 0 {
        // Relocate fdt
        let fdt_addr = relocate_fdt(fdt_addr);
        unsafe { FDT_RELOCATED_ADDR = fdt_addr };
        FDT_RELOCATE_TOKEN.store(true, Ordering::Release);
        fdt_addr
    } else {
        while !FDT_RELOCATE_TOKEN.load(Ordering::Acquire) {
            spin_loop_hint();
        }

        unsafe { FDT_RELOCATED_ADDR }
    };

    let fdt = unsafe { fdt::FDT::from_raw(fdt_addr) }.unwrap();

    // Initialize platform
    unsafe {
        core::ptr::write(
            mem::data(hartid).platform.as_mut_ptr(),
            PLATFORM::new(hartid, fdt),
        );
    }

    if hartid == 0 {
        // Print MOTD
        mprint!(include_str!("./motd.txt")).unwrap();

        crate::mprintln!("FDT relocated to 0x{:016X}", fdt_addr as usize).unwrap();

        // Setup pmp
        setup_pmp();
    }

    // Setup mtvec
    trap::setup();

    // Fixup fdt
    if hartid != 0 {
        while !FDT_FIXMP_TOKEN.load(Ordering::Acquire) {
            spin_loop_hint();
        }
    } else {
        fixup_fdt(fdt_addr);
        FDT_FIXMP_TOKEN.store(true, Ordering::Release);
    }

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

fn relocate_fdt(original: *const u8) -> *mut u8 {
    let parsed = unsafe { fdt::FDT::from_raw(original) }.unwrap();
    let size = parsed.total_size();
    unsafe {
        core::ptr::copy_nonoverlapping(original, &mut FDT_STORAGE[0] as _, size as usize);
    }

    unsafe { &mut FDT_STORAGE[0] as _ }
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
    let fw_start = 0x0000_0000;
    let fw_size = 0x20_0000;

    let addr_encoded = (fw_start >> 2) | ((fw_size >> 3) - 1);

    crate::mprintln!("PMP Encoded: {:016X}", addr_encoded).unwrap();

    riscv::register::pmpaddr0::write(addr_encoded);
    riscv::register::pmpaddr1::write((1 << (56-3)) - 1); // Maximum range
    riscv::register::pmpcfg0::write(
        (
            (3 << 3) | 1 // NAPOT + not locking + can read (for FDT)
        ) | (
            ((3 << 3) | 7) << 8
        )
    );
}
