use core::mem::MaybeUninit;
use core::sync::atomic::*;
use core::cell::UnsafeCell;
use crate::ipi::*;

pub struct HartData {
    pub ipi_req: UnsafeCell<IPIReq>,
    pub ipi_pending: AtomicBool,
    pub platform: MaybeUninit<crate::PLATFORM>,
}

impl HartData {
    pub const fn new() -> Self {
        HartData {
            ipi_req: UnsafeCell::new(IPIReq::S_IPI), // Random value
            ipi_pending: AtomicBool::new(false),
            platform: MaybeUninit::uninit(),
        }
    }

    // This function assumes that the platform is properly initialized during the entry of crate::boot
    pub fn platform(&mut self) -> &mut crate::PLATFORM {
        unsafe {
            &mut * self.platform.as_mut_ptr()
        }
    }

    pub fn ipi_set(&self, req: IPIReq) {
        unsafe { *self.ipi_req.get() = req };

        self.ipi_pending.store(true, Ordering::Release);
    }

    pub fn ipi_wait(&self) {
        while self.ipi_pending.load(Ordering::Acquire) {
            spin_loop_hint();
        }
    }

    pub fn ipi_handle(&mut self) {
        use crate::platform::PlatformOps;

        while self.ipi_pending.load(Ordering::Acquire) == false {
            spin_loop_hint();
        }

        let req = unsafe { *self.ipi_req.get() };
        crate::ipi::handle_ipi(req);
        self.platform().clear_ipi();
        self.ipi_pending.store(false, Ordering::Release);
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

/*
fn _assert_storage_size() {
    unsafe {
        core::mem::transmute::<AllStorage, [u8; crate::HART_STORE_SIZE * crate::HART_CNT]>(
            [HartStorage::new(); crate::HART_CNT],
        );
    }
}
*/

pub fn data(hartid: usize) -> &'static mut HartData {
    unsafe { &mut STORAGE[hartid].data }
}

pub fn local_data() -> &'static mut HartData {
    let hid = riscv::register::mhartid::read();
    data(hid)
}
