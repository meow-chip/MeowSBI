use core::mem::MaybeUninit;
use core::sync::atomic::*;

pub struct HartData {
    pub ipi_req: AtomicUsize,
    pub ipi_lock: AtomicBool,
    pub platform: MaybeUninit<crate::PLATFORM>,
}

impl HartData {
    pub const fn new() -> Self {
        HartData {
            ipi_req: AtomicUsize::new(0),
            ipi_lock: AtomicBool::new(false),
            platform: MaybeUninit::uninit(),
        }
    }

    // This function assumes that the platform is properly initialized during the entry of crate::boot
    pub fn platform(&mut self) -> &mut crate::PLATFORM {
        unsafe {
            &mut * self.platform.as_mut_ptr()
        }
    }
}

pub struct HartStack<const STACK_SIZE: usize> {
    _inner: [MaybeUninit<u8>; STACK_SIZE],
}

impl<const STACK_SIZE: usize> HartStack<STACK_SIZE> {
    const fn new() -> Self {
        Self {
            _inner: [MaybeUninit::uninit(); STACK_SIZE],
        }
    }
}

#[repr(C)] // Ensures that stack lies in the tail of this struct
pub struct HartStorage<const STORE_SIZE: usize> {
    pub data: HartData,
    pub stack: HartStack<{ STORE_SIZE - core::mem::size_of::<HartData>() }>,
}

impl<const STORE_SIZE: usize> HartStorage<STORE_SIZE> {
    const fn new() -> Self {
        Self {
            data: HartData::new(),
            stack: HartStack::new(),
        }
    }
}

type AllStorage = [HartStorage<{ crate::HART_STORE_SIZE }>; crate::HART_CNT];

#[link_section = ".data"]
pub static mut STORAGE: AllStorage = [HartStorage::new(); crate::HART_CNT];

fn _assert_storage_size() {
    unsafe {
        core::mem::transmute::<AllStorage, [u8; crate::HART_STORE_SIZE * crate::HART_CNT]>(
            [HartStorage::new(); crate::HART_CNT],
        );
    }
}

pub fn data(hartid: usize) -> &'static mut HartData {
    unsafe { &mut STORAGE[hartid].data }
}

pub fn local_data() -> &'static mut HartData {
    let hid = riscv::register::mhartid::read();
    data(hid)
}
