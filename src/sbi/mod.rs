const SBI_IMPL_ID: usize = 0x776f654d; // Meow in little endian
const SBI_IMPL_VERSION: usize = 0x1;

const SBI_SPEC_MAJOR: usize = 0;
const SBI_SPEC_MINOR: usize = 2;

#[allow(dead_code)]
#[repr(usize)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum SBIExt {
    SetTimer = 0x00,
    ConsolePutChar = 0x01,
    ConsoleGetChar = 0x02,
    ClearIPI = 0x03,
    SendIPI = 0x04,
    RemoteFENCE_I = 0x05,
    RemoteSFENCE_VMA = 0x06,
    RemoteSFENCE_VMA_ASID = 0x07,
    Shutdown = 0x08,

    Base = 0x10,

    IPI = 0x735049,
    RFENCE = 0x52464E43,
    TIME = 0x54494D45,
    // Sorry, no HSM
}

const ALL_SBI_EXT: [SBIExt; 13] = [
    SBIExt::SetTimer,
    SBIExt::ConsolePutChar,
    SBIExt::ConsoleGetChar,
    SBIExt::ClearIPI,
    SBIExt::SendIPI,
    SBIExt::RemoteFENCE_I,
    SBIExt::RemoteSFENCE_VMA,
    SBIExt::RemoteSFENCE_VMA_ASID,
    SBIExt::Shutdown,
    SBIExt::Base,
    SBIExt::IPI,
    SBIExt::RFENCE,
    SBIExt::TIME,
];

#[repr(usize)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum SBIBaseFunc {
    GetSBISpecVersion = 0x0,
    GetSBIImplID = 0x1,
    GetSBIImplVersion = 0x2,
    ProbExtension = 0x3,
    GetMVENDROID = 0x4,
    GetMARCHID = 0x5,
    GetMIMPLID = 0x6,
}

#[repr(isize)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum SBIErr {
    Success = 0,
    Legacy = 1,
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
}

pub struct SBIRet {
    pub error: SBIErr,
    pub value: usize,
}

impl From<usize> for SBIRet {
    fn from(v: usize) -> Self {
        Self {
            error: SBIErr::Success,
            value: v,
        }
    }
}

impl From<SBIErr> for SBIRet {
    fn from(e: SBIErr) -> Self {
        Self { error: e, value: 0 }
    }
}

impl From<Option<usize>> for SBIRet {
    fn from(v: Option<usize>) -> Self {
        match v {
            None => SBIErr::NotSupported.into(),
            Some(v) => v.into(),
        }
    }
}

pub fn call(ext: usize, func: usize, a0: usize, a1: usize, a2: usize) -> SBIRet {
    // FIXME: hart_mask_base
    let ext = unsafe { core::mem::transmute(ext) };
    // crate::mprintln!("[MeowSBI] SBI Call: {:?}", ext).unwrap();
    match ext {
        SBIExt::Base => {
            if func > SBIBaseFunc::GetMIMPLID as _ {
                SBIErr::NotSupported.into()
            } else {
                let func = unsafe { core::mem::transmute::<_, SBIBaseFunc>(func) };
                match func {
                    SBIBaseFunc::GetSBISpecVersion => {
                        ((SBI_SPEC_MAJOR << 24) | (SBI_SPEC_MINOR)).into()
                    }
                    SBIBaseFunc::GetSBIImplID => SBI_IMPL_ID.into(),
                    SBIBaseFunc::GetSBIImplVersion => SBI_IMPL_VERSION.into(),
                    SBIBaseFunc::ProbExtension => {
                        for ext in &ALL_SBI_EXT {
                            if *ext as usize == a0 {
                                return SBIErr::Success.into();
                            }
                        }
                        return SBIErr::NotSupported.into();
                    }
                    SBIBaseFunc::GetMVENDROID => {
                        riscv::register::mvendorid::read().map(|e| e.bits()).into()
                    }
                    SBIBaseFunc::GetMARCHID => {
                        riscv::register::marchid::read().map(|e| e.bits()).into()
                    }
                    SBIBaseFunc::GetMIMPLID => {
                        riscv::register::mimpid::read().map(|e| e.bits()).into()
                    }
                }
            }
        }
        SBIExt::ConsolePutChar => {
            crate::serial::putc(a0 as u8);
            0usize.into()
        }
        SBIExt::ConsoleGetChar => SBIRet {
            error: SBIErr::Legacy,
            value: crate::serial::getc() as usize,
        },
        SBIExt::SetTimer => set_timer(a0),
        SBIExt::ClearIPI => unsafe {
            riscv::register::mip::clear_ssoft();
            0usize.into()
        },
        SBIExt::SendIPI => ipi_ptr(a0, crate::ipi::IPIReq::S_IPI),
        SBIExt::RemoteFENCE_I => ipi_ptr(a0, crate::ipi::IPIReq::FENCE_I),
        SBIExt::RemoteSFENCE_VMA => ipi_ptr(a0, crate::ipi::IPIReq::SFENCE_VMA),
        SBIExt::RemoteSFENCE_VMA_ASID => ipi(
            unsafe { *(a0 as *const usize) },
            0,
            crate::ipi::IPIReq::SFENCE_VMA,
        ),
        SBIExt::Shutdown => loop {
            core::sync::atomic::spin_loop_hint()
        }, // TOOD: proper impl

        SBIExt::IPI => {
            if func != 0 {
                SBIErr::NotSupported.into()
            } else {
                ipi(a0, a1, crate::ipi::IPIReq::S_IPI)
            }
        }
        SBIExt::RFENCE => match func {
            0 => ipi(a0, a1, crate::ipi::IPIReq::FENCE_I),
            1 | 2 => ipi(a0, a1, crate::ipi::IPIReq::SFENCE_VMA),
            _ => SBIErr::NotSupported.into(),
        },
        SBIExt::TIME => {
            if func != 0 {
                SBIErr::NotSupported.into()
            } else {
                set_timer(a0)
            }
        }
    }
}

fn set_timer(timer: usize) -> SBIRet {
    use crate::platform::PlatformOps;
    crate::mem::local_data().platform().set_timer(timer as u64);

    // TODO: mtip may show a false postive (when setting a larger mtimecmp), and stimer may never be cleared
    let mtip = riscv::register::mip::read().mtimer();
    if mtip {
        unsafe {
            riscv::register::mie::clear_mtimer();
            riscv::register::mip::set_stimer();
        }
    } else {
        unsafe {
            riscv::register::mie::set_mtimer();
            riscv::register::mip::clear_stimer();
        }
    }
    0usize.into()
}

fn ipi(mask: usize, base: usize, ipi: crate::ipi::IPIReq) -> SBIRet {
    // TODO: handles HART_COUNT > 64
    let mask = if base == core::usize::MAX {
        core::usize::MAX
    } else {
        mask << base
    };

    crate::ipi::send_ipi(mask, ipi);
    0usize.into()
}

fn ipi_ptr(p: usize, i: crate::ipi::IPIReq) -> SBIRet {
    let mut mask = core::usize::MAX;
    if p != 0 {
        // MPRV = 1, then read
        unsafe {
            llvm_asm!(r#"
            li t0, (1<<17)
            mv t1, $0
            csrrs t0, mstatus, t0
            lw t1, 0(t1)
            csrw mstatus, t0
            mv $1, t1
            "# : "=r"(mask) : "r"(p) :: "volatile");
        }
    }

    ipi(mask, 0, i)
}
