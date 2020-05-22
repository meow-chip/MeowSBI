use std::fs;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let payload = env::var("MEOWSBI_PAYLOAD").ok();

    let payload_rs = Path::new(&out_dir).join("payload_content.rs");

    if let Some(payload) = payload {
        fs::write(
            &payload_rs,
            format!(r#"
            global_asm!("
                .section .payload, \"ax\", %progbits
                .globl payload
            payload:
                .incbin \"{}\"
            ");"#, payload),
        ).unwrap();
    } else {
        fs::write(&payload_rs, r#"

        #[cfg(feature = "payload")]
        compile_error!("MEOWSBI_PAYLOAD environment variable not set!");
        
        "#).unwrap();
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-var-changed=MEOWSBI_PAYLOAD");
}
