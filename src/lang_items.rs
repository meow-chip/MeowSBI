use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}


#[no_mangle]
fn abort() -> ! {
    panic!("abort called");
}
