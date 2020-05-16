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
        let memask = 0xFFFF - (1 << 9);
        llvm_asm!("csrw medeleg, $0" :: "r"(memask) :: "volatile");

        // Setup MIDELEG
        riscv::register::mideleg::set_sext();
        riscv::register::mideleg::set_ssoft();
        riscv::register::mideleg::set_stimer();
        // TODO: U-mode interrupts

        // Setup MEDELEG
        let medeleg = 0xFFFF & !(1<<9); // Delegate everything except S_CALL
        llvm_asm!("csrw medeleg, $0" :: "r"(medeleg) :: "volatile"); 

        // Setup MTVEC
        // let addr = trap_enter as usize;
        // llvm_asm!("csrw mtvec, $0" :: "r"(addr) :: "volatile");
        riscv::register::mtvec::write(
            trap_enter as usize,
            riscv::register::mtvec::TrapMode::Direct,
        );

        // Set corresponding MIE
        riscv::register::mie::set_mext();
        riscv::register::mie::set_msoft();
        // riscv::register::mie::set_mtimer();
        // Mtimer gets enabled on first set_timer

        // Sets MPIE
        riscv::register::mstatus::set_mpie();
    }
}

macro_rules! op_reg {
    ($op: literal, $cnt:literal) => {
        llvm_asm!(concat!(
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
    () => {
        32 * 8
    };
}

fn _tf_size_check(tf: TrapFrame) -> [u8; TF_SIZE!()] {
    unsafe { core::mem::transmute(tf) }
}

// Generates a trap frame on the top of the stack, then call wrapped_trap
#[naked]
#[link_section = ".text.trap"]
unsafe fn trap_enter() -> ! {
    llvm_asm!("csrrw sp, mscratch, sp"); // Swap machine and friends
    llvm_asm!("bnez sp, 1f");
    llvm_asm!("csrr sp, mscratch");

    llvm_asm!(concat!("1: addi sp, sp, -", stringify!(32 * 8)));
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

    llvm_asm!("csrrw t0, mscratch, x0");
    llvm_asm!("sd t0, 8*2(sp)");

    llvm_asm!("mv a0, sp");
    llvm_asm!("call wrapped_trap");

    trap_ret();
}

#[naked]
// Assumes that the top of the stack is the TrapFrame
unsafe fn trap_ret() -> ! {
    llvm_asm!("csrr t0, mstatus");
    llvm_asm!("srli t0, t0, 11");
    llvm_asm!("andi t0, t0, 3"); // Now t0 = MPP

    llvm_asm!("xori t0, t0, 3");
    llvm_asm!("beqz t0, 1f"); // MPP = M
    llvm_asm!(concat!("addi t0, sp, ", stringify!(32 * 8)));
    llvm_asm!("csrw mscratch, t0");
    llvm_asm!("1: ");

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
    llvm_asm!("mret");

    unreachable!()
}

#[naked]
pub unsafe fn next_ret(hartid: usize, fdt_addr: *const u8) -> ! {
    llvm_asm!("mv s0, sp");
    llvm_asm!(concat!("addi sp, sp, -", stringify!(32 * 8)));
    llvm_asm!("1: addi s0, s0, -8");
    llvm_asm!("sd x0, 0(s0)");
    llvm_asm!("bne sp, s0, 1b");

    llvm_asm!("sd $0, 8*10(sp)" :: "r"(hartid) :: "volatile");
    llvm_asm!("sd $0, 8*11(sp)" :: "r"(fdt_addr) :: "volatile");

    trap_ret();
}

#[no_mangle]
#[link_name = "wrapped_trap"]
pub extern "C" fn wrapped_trap<'a>(tf: &'a mut TrapFrame) {
    use riscv::register::mcause::{read, Exception, Interrupt, Trap};
    let mcause = read();

    match mcause.cause() {
        Trap::Exception(Exception::SupervisorEnvCall) => {
            let ret = crate::sbi::call(
                tf.reg[17], // a7
                tf.reg[16], // a6
                tf.reg[10], tf.reg[11], tf.reg[12],
            );

            if ret.error == crate::sbi::SBIErr::Legacy {
                tf.reg[10] = ret.value as _;
            } else {
                tf.reg[10] = ret.error as _;
                tf.reg[11] = ret.value as _;
            }

            // Increment MEPC
            let mepc = riscv::register::mepc::read();
            riscv::register::mepc::write(mepc + 4); // We don't have C.ECALL
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            // Sets S-Timer bit in mip
            unsafe {
                riscv::register::mip::set_stimer();
                riscv::register::mie::clear_mtimer();
            }
        }
        _ => todo!(),
    }
}
