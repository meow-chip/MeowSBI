use core::sync::atomic::*;
use core::mem::MaybeUninit;


pub struct HartData {
    pub ipi_req: AtomicUsize,
    pub ipi_lock: AtomicBool,
}

impl HartData {
    pub const fn new() -> Self {
        HartData {
            ipi_req: AtomicUsize::new(0),
            ipi_lock: AtomicBool::new(false),
        }
    }
}

pub struct HartStack<const STACK_SIZE: usize> {
    inner: [MaybeUninit<u8>; STACK_SIZE],
}

impl<const STACK_SIZE: usize> HartStack<STACK_SIZE> {
    const fn new() -> Self {
        Self {
            inner: [MaybeUninit::uninit(); STACK_SIZE],
        }
    }

    pub unsafe fn start_ptr(&self) -> *const u8 {
        self.inner[STACK_SIZE-1].as_ptr()
    }
}

#[repr(C)] // Ensures that stack lies in the tail of this struct
pub struct HartStorage<const STORE_SIZE: usize> {
    pub data: HartData,
    pub stack: HartStack<{STORE_SIZE - core::mem::size_of::<HartData>()}>,
}

impl<const STORE_SIZE: usize> HartStorage<STORE_SIZE> {
    const fn new() -> Self {
        Self {
            data: HartData::new(),
            stack: HartStack::new(),
        }
    }
}

type AllStorage = [HartStorage<{crate::HART_STORE_SIZE}>; crate::HART_CNT];

#[link_section = ".data"]
pub static STORAGE: AllStorage = [HartStorage::new(); crate::HART_CNT];


fn _assert_storage_size() {
    unsafe {
        core::mem::transmute::<AllStorage, [u8; crate::HART_STORE_SIZE * crate::HART_CNT]>(
            [HartStorage::new(); crate::HART_CNT]
        );
    }
}
