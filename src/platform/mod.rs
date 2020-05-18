pub mod meowv64;
pub mod qemu;

pub trait PlatformOps: Sized {
    fn new(hardid: usize, fdt: fdt::FDT) -> Self;
    fn early_init(&self, _cold: bool) {}
    fn final_init(&self, _cold: bool) {}

    fn set_timer(&self, instant: u64);

    fn put_char(&self, c: u8);
    fn get_char(&self) -> u8;

    fn send_ipi(&self, hartid: usize);
    fn clear_ipi(&self);
}
