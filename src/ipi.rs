use core::sync::atomic::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IPIReq {
    S_IPI, // S-mode IPI
    FENCE_I,
    SFENCE_VMA,
}

static LOCK: AtomicBool = AtomicBool::new(false);

pub fn send_ipi(mask: usize, req: IPIReq) {
    let started_mask = (1 << crate::HART_CNT) - 1;
    let mask = mask & started_mask;

    // Acquire lock
    while LOCK.swap(true, Ordering::Acquire) {
        spin_loop_hint();
        if riscv::register::mip::read().msoft() {
            // Other core triggered an IPI, handle right now
            crate::mem::local_data().ipi_handle();
        }
    }

    let cur_hart = riscv::register::mhartid::read();
    let mut sending = mask;
    let mut waiting = sending;

    let platform = crate::mem::local_data().platform();
    use crate::platform::PlatformOps;

    while sending != 0 {
        let current = sending.trailing_zeros() as usize;

        if current == cur_hart {
            handle_ipi(req);
        } else {
            crate::mem::data(current).ipi_set(req);
            platform.send_ipi(current);
        }

        sending = sending ^ (1<<current);
    }

    while waiting != 0 {
        let current = waiting.trailing_zeros() as usize;

        if current == cur_hart {
            // Does nothing
        } else {
            crate::mem::data(current).ipi_wait();
        }

        waiting = waiting ^ (1<<current);
    }

    LOCK.store(false, Ordering::Release);
}

pub fn handle_ipi(req: IPIReq) {
    match req {
        IPIReq::S_IPI => unsafe { riscv::register::mip::set_ssoft() },
        IPIReq::FENCE_I => unsafe { llvm_asm!("FENCE.I") },
        IPIReq::SFENCE_VMA => unsafe { riscv::asm::sfence_vma_all() },
    }
}
