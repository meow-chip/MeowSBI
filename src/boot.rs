#[no_mangle]
#[link_section = ".text.entry"]
#[naked]
pub unsafe extern "C" fn entry() -> ! {
    // TODO: relocate self

    // Setup registers
    setup_register();
    setup_stack();
    llvm_asm!("la a2, payload");
    llvm_asm!("j boot");
    unreachable!();
}

#[naked]
unsafe fn setup_stack() {
    // a0 and a1 is perserved
    llvm_asm!("mv s0, a0");
    llvm_asm!("mv s1, a1");

    llvm_asm!("addi t0, a0, 1");
    llvm_asm!(concat!("slli t0, t0, ", crate::HART_STORE_SHIFT_STR!()));
    llvm_asm!("mv sp, $0" :: "r"(
        &crate::mem::STORAGE as *const _ as usize
    ) :: "volatile");
    llvm_asm!("add sp, sp, t0");

    llvm_asm!("mv a0, s0");
    llvm_asm!("mv a1, s1");
}

macro_rules! clear_reg {
    ($x:ident) => {
        llvm_asm!(concat!("li ", stringify!($x), ", 0"));
    };
}

#[naked]
unsafe fn setup_register() {
    // FENCE.I
    llvm_asm!("fence.i");

    // Clear everything except ra, a0(hartid) and a1(fdt addr)
    clear_reg!(sp);
    clear_reg!(gp);
    clear_reg!(tp);
    clear_reg!(t0);
    clear_reg!(t1);
    clear_reg!(t2);
    clear_reg!(s0);
    clear_reg!(s1);
    // a0 is hartid
    // a1 is fdt addr
    clear_reg!(a2);
    clear_reg!(a3);
    clear_reg!(a4);
    clear_reg!(a5);
    clear_reg!(a6);
    clear_reg!(a7);
    clear_reg!(s2);
    clear_reg!(s3);
    clear_reg!(s4);
    clear_reg!(s5);
    clear_reg!(s6);
    clear_reg!(s7);
    clear_reg!(s8);
    clear_reg!(s9);
    clear_reg!(s10);
    clear_reg!(s11);
    clear_reg!(t3);
    clear_reg!(t4);
    clear_reg!(t5);
    clear_reg!(t6);
}
