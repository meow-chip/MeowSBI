pub mod meowv64;
pub mod qemu;

pub trait PlatformOps: Sized {
    fn on(hardid: usize) -> Self;
    fn early_init(&self, _cold: bool) {}
    fn final_init(&self, _cold: bool) {}

    fn set_timer(&self, instant: u64);

    fn put_char(&self, c: u8);
    fn get_char(&self) -> u8;

    fn local() -> Self {
        Self::on(riscv::register::mhartid::read())
    }
}
