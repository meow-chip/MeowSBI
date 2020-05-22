extern "C" {
    pub fn _payload_start();
    pub fn _payload_end();
}

include!(concat!(env!("OUT_DIR"), "/payload_content.rs"));

pub const HAS_PAYLOAD: bool = cfg!(feature = "payload");
pub const PAYLOAD_TARGET: *mut u8 = 0x80200000usize as _;

pub fn relocate(payload_addr: *const u8) {
    if !HAS_PAYLOAD {
        crate::mprintln!("MeowSBI built without payload, skipping payload relocation").unwrap();
        return;
    }

    crate::mprintln!("Payload relocation 0x{:016X} -> 0x{:016X}", payload_addr as usize, PAYLOAD_TARGET as usize).unwrap();
    unsafe {
        core::ptr::copy(payload_addr, PAYLOAD_TARGET, _payload_end as usize - _payload_start as usize);
    }
    crate::mprintln!("Relocation complete").unwrap();
}
