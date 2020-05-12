use riscv;

/**
 * Trap frame
 * we are only going to save registers, because we will keep supervisor state CSR unchanged
 */
pub struct TrapFrame {
    pub reg: [usize; 32],
}

pub fn setup() {
    unsafe {
        // Setup MEDELEG, only handles S_CALL for now
        let memask = 0xFFFF - (1<<9);
        asm!("csrw medeleg, $0" :: "r"(memask) :: "volatile");

        // Setup MIDELEG
        riscv::register::mideleg::set_sext();
        riscv::register::mideleg::set_ssoft();
        riscv::register::mideleg::set_stimer();
        // TODO: U-mode interrupts

        // Setup MTVEC
        // let addr = trap_enter as usize;
        // asm!("csrw mtvec, $0" :: "r"(addr) :: "volatile");
        riscv::register::mtvec::write(trap_enter as usize, riscv::register::mtvec::TrapMode::Direct);

        // Set corresponding MIE
        riscv::register::mie::set_mext();
        riscv::register::mie::set_msoft();
        riscv::register::mie::set_mtimer();

        // Sets MPIE
        riscv::register::mstatus::set_mpie();
    }
}

macro_rules! op_reg {
    ($op: literal, $cnt:literal) => {
        asm!(concat!(
            $op,
            " x",
            stringify!($cnt),
            ", 8*",
            stringify!($cnt),
            "(sp)"
        ))
    };
}

macro_rules! store_reg {
    ($cnt: literal) => {
        op_reg!("sd", $cnt);
    };
}

macro_rules! load_reg {
    ($cnt: literal) => {
        op_reg!("ld", $cnt);
    };
}

macro_rules! TF_SIZE {
    () => (32*8);
}

const TF_SIZE_CONST: usize = TF_SIZE!();

fn _tf_size_check(tf: TrapFrame) -> [u8; TF_SIZE_CONST] {
    unsafe {
        core::mem::transmute(tf)
    }
}

// Generates a trap frame on the top of the stack, then call wrapped_trap
#[naked]
#[link_section = ".text.trap"]
unsafe fn trap_enter() -> ! {
    asm!("csrrw sp, mscratch, sp"); // Swap machine and friends
    asm!("bnez sp, 1f");
    asm!("csrr sp, mscratch");

    asm!(concat!("1: addi sp, sp, -", stringify!(32*8)));
    store_reg!(1);
    store_reg!(3);
    store_reg!(4);
    store_reg!(5);
    store_reg!(6);
    store_reg!(7);
    store_reg!(8);
    store_reg!(9);
    store_reg!(10);
    store_reg!(11);
    store_reg!(12);
    store_reg!(13);
    store_reg!(14);
    store_reg!(15);
    store_reg!(16);
    store_reg!(17);
    store_reg!(18);
    store_reg!(19);
    store_reg!(20);
    store_reg!(21);
    store_reg!(22);
    store_reg!(23);
    store_reg!(24);
    store_reg!(25);
    store_reg!(26);
    store_reg!(27);
    store_reg!(28);
    store_reg!(29);
    store_reg!(30);
    store_reg!(31);

    asm!("csrrw t0, mscratch, x0");
    asm!("sd t0, 8*2(sp)");

    asm!("mv a0, sp");
    asm!("call wrapped_trap");

    trap_ret();
}

#[naked]
// Assumes that the top of the stack is the TrapFrame
unsafe fn trap_ret() -> ! {
    asm!("csrr t0, mstatus");
    asm!("srli t0, t0, 11");
    asm!("andi t0, t0, 3"); // Now t0 = MPP

    asm!("xori t0, t0, 3");
    asm!("beqz t0, 1f"); // MPP = M
    asm!(concat!("addi t0, sp, ", stringify!(32*8)));
    asm!("csrw mscratch, t0");
    asm!("1: ");

    load_reg!(1);
    load_reg!(3);
    load_reg!(4);
    load_reg!(5);
    load_reg!(6);
    load_reg!(7);
    load_reg!(8);
    load_reg!(9);
    load_reg!(10);
    load_reg!(11);
    load_reg!(12);
    load_reg!(13);
    load_reg!(14);
    load_reg!(15);
    load_reg!(16);
    load_reg!(17);
    load_reg!(18);
    load_reg!(19);
    load_reg!(20);
    load_reg!(21);
    load_reg!(22);
    load_reg!(23);
    load_reg!(24);
    load_reg!(25);
    load_reg!(26);
    load_reg!(27);
    load_reg!(28);
    load_reg!(29);
    load_reg!(30);
    load_reg!(31);

    load_reg!(2);
    asm!("mret");

    unreachable!()
}

#[naked]
pub unsafe fn next_ret() -> ! {
    asm!("mv s0, sp");
    asm!(concat!("addi sp, sp, -", stringify!(32*8)));
    asm!("1: addi s0, s0, -8");
    asm!("sd x0, 0(s0)");
    asm!("bne sp, s0, 1b");

    asm!("csrr t0, mhartid");
    asm!("sd t0, 8*10(sp)");

    trap_ret();
}

#[no_mangle]
#[link_name = "wrapped_trap"]
pub extern "C" fn wrapped_trap<'a>(tf: &'a mut TrapFrame) {
    use riscv::register::mcause::*;
    let mcause = read();

    match mcause.cause() {
        Trap::Exception(Exception::SupervisorEnvCall) => {
            let ret = crate::sbi::call(
                tf.reg[17], // a7
                tf.reg[16], // a6
                tf.reg[10], tf.reg[11], tf.reg[12],
            );
            
            tf.reg[10] = ret.error as _;
            tf.reg[11] = ret.value as _;

            // Increment MEPC
            let mepc = riscv::register::mepc::read();
            riscv::register::mepc::write(mepc + 4); // We don't have C.ECALL
        },
        _ => todo!(),
    }
}
