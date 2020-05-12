#[no_mangle]
#[link_section = ".text.entry"]
#[export_name = "entry"]
#[naked]
pub extern "C" fn entry() -> ! {
    // Setup registers
    setup_register();
    setup_stack();
    crate::boot();
}

#[naked]
extern "C" fn setup_stack() {
    unsafe {
        asm!("csrr t0, mhartid");
        asm!("addi t0, t0, 1");
        asm!(concat!("slli t0, t0, ", crate::HART_STORE_SHIFT_STR!()));
        asm!("mv sp, $0" :: "r"(
            &crate::mem::STORAGE as *const _ as usize
        ) :: "volatile");
        asm!("add sp, sp, t0");
    }
}

macro_rules! clear_reg {
    ($x:ident) => {
        asm!(concat!("li ", stringify!($x), ", 0"));
    };
}

#[naked]
fn setup_register() {
    unsafe {
        // FENCE.I
        asm!("fence.i");

        // Clear everything except ra, so it can return
        clear_reg!(sp);
        clear_reg!(gp);
        clear_reg!(tp);
        clear_reg!(t0);
        clear_reg!(t1);
        clear_reg!(t2);
        clear_reg!(s0);
        clear_reg!(s1);
        clear_reg!(a0);
        clear_reg!(a1);
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
}
