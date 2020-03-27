trait PlatformOps {
    fn early_init(cold: bool);
    fn final_init(cold: bool);

    fn set_timer(instant: u64);
    fn get_timer() -> u64;
}

struct Platform<P: PlatformOps> {
    _pd: core::marker::PhantomData<P>,
}
