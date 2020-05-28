# How to run rCore on your SoC (RISC-V 64)

## Prerequisites

Your SoC should at least has the following devices connected to the memory bus:

- Memory, at least 2M of RAM should be expected
- An NS8250 compatible UART
- CLINT and PLIC
  - CLINT with standard SiFive memory mapping
  - PLIC context 2*i+1 = hart i S-mode

Your RISC-V core should at least supports the following ISA:

- RV64IMACSU

## Steps

First, you need to compile rCore. Then you will need to pick a compatible M-mode firmware.

### Compiling

rCore currently only supports virtio-blk block devices, so we have to embed an userspace image into the rCore binary.

Go to the /user path and issue the following command:

```bash
make sfsimg arch=riscv64 prebuilt=1
```

rCore will download an prebuilt userspace file system image for us.

Then, you need apply the following patch to prevent changing read-only file system image:

```diff
--- a/kernel/src/fs/mod.rs
+++ b/kernel/src/fs/mod.rs
@@ -89,17 +90,20 @@ lazy_static! {
         devfs.add("fb0", Arc::new(Fbdev::default())).expect("failed to mknod /dev/fb0");
 
         // mount DevFS at /dev
-        let dev = root.find(true, "dev").unwrap_or_else(|_| {
-            root.create("dev", FileType::Dir, 0o666).expect("failed to mkdir /dev")
-        });
-        dev.mount(devfs).expect("failed to mount DevFS");
+        let dev = root.find(true, "dev");
+        if let Ok(dev) = dev {
+            dev.mount(devfs).expect("failed to mount DevFS");
+        } else {
+            info!("no /dev, skipping");
+        }
 
         // mount RamFS at /tmp
         let ramfs = RamFS::new();
-        let tmp = root.find(true, "tmp").unwrap_or_else(|_| {
-            root.create("tmp", FileType::Dir, 0o666).expect("failed to mkdir /tmp")
-        });
-        tmp.mount(ramfs).expect("failed to mount RamFS");
+        if let Ok(tmp) = root.find(true, "tmp") {
+            tmp.mount(ramfs).expect("failed to mount RamFS");
+        } else {
+            info!("no /tmp, skipping");
+        }
 
         root
     };
```

Depending on the version of rCore you use, you may also want to edit the linker script to zero out the `.sbss` segment on boot:

```diff
--- a/kernel/src/arch/riscv/board/u540/linker.ld
+++ b/kernel/src/arch/riscv/board/u540/linker.ld
@@ -42,11 +42,14 @@ SECTIONS
         *(.bss.stack)
     }
 
+    sbss = .;
     .bss : {
-        sbss = .;
         *(.bss .bss.*)
-        ebss = .;
     }
+    .sbss : {
+        *(.sbss .sbss.*)
+    }
+    ebss = .;
 
     PROVIDE(end = .);
 }
```

Finally, you may want to configure constants related to your memory setup. See `kernel/src/arch/riscv/constants.rs`. Most imporantly, change the default kernel heap size to make sure it doesn't exceed your DRAM capacity.

Issue the following command within the `kernel` path:

```bash
make ARCH=riscv64 FEATURES=link_user MODE=release SMP=[YOUR HART CNT]
```

the generated raw binary will be at `kernel/target/riscv64/release/kernel.img`

### Firmware
rCore (currently) expects SBI 0.1 environment, so the following options are available:

- OpenSBI (recommended, >= 0.6)
- MeowSBI (use at your own risk)

#### OpenSBI
You need to add your platform into the /platform directory within the OpenSBI source code tree. You may start from copying from the QEMU platform, located at /platform/generic

Make sure to configure the following parameters:
- Where to put the relocated FDT, because it's traditionally set at 0x82200000 phys mem, or DRAM_BASE+ 0x2200000, but most SoC don't have such a large memory space.
- Where to link the payload. It defaults to 0x80200000, and OpenSBI itself uses about 1M RAM, so if you are low on RAM (SoC only have 2M-4M RAM, etc.), you may change this parameter to leave more space for rCore.
- If your DRAM_BASE =/= 0x80000000, make sure to change all hard-coded memory addresses.

Also, if your SoC / core doesn't provide FSB with a copy of the device tree, you may have to embed a flatten device tree into the firmware. See the device tree section.

Compile with FW_PAYLOAD mode to embed rCore into the firmware ELF, OpenSBI will generate it's output to /build/platform/[YOUR PLATFORM NAME]/firmware/fw_payload.bin

### MeowSBI
Generally the workflow is similar. You may need to change the following constants:

- src/main.rs: hart counts, stack space per hart, etc.
- src/serial.rs: early print setup
- src/provided/dt.fdt: the embedded flatten device tree. If no device tree is provided to MeowSBI, the embedded one will be used.

To embed a payload, set the environment variable `MEOWSBI_PAYLOAD` to the path of the raw kernel image, then:

```bash
cargo build --target=riscv64gc-unknown-none-elf --features="payload" --release 
```

The generated ELF will be at `./target/riscv64gc-unknown-none-elf/release/meow-sbi`

### Device tree
See https://github.com/devicetree-org/devicetree-specification for specification.

The device tree is expected to contain:

- An `/aliases` node, containing a property `serial0` pointing to the default serial device.
- An `/choosen` node, containing a property `stdout-path`, containing the path of the default serial device.
- An default serial device, compatible set to `ns16550a`, with interrupt setups.
- An `/soc` node, containing the PLIC and CLINT node
- An `/cpus` node, containing specifications of RISC-V harts.
- An `/memory` node for DRAM base and size.


Example device tree:
```dts
/dts-v1/;

/ {
  #address-cells = <0x2>;
  #size-cells = <0x2>;

  compatible = "riscv-meowv64";
  model = "riscv-meowv64,meowv64";

  aliases {
    serial0 = &serial;
  };

  chosen {
    stdout-path = "/uart@10000000";
  };

  serial: uart@10000000 {
    interrupts = <0x1>;
    interrupt-parent = < &PLIC >;
    clock-frequency = <11059200>;
    current-speed = <115200>;
    reg = <0x0 0x10000000 0x00000000 0x00000100>;
    reg-offset=<0x1000>;
    reg-shift=<2>;
    compatible = "ns16550a";
  };

  memory@80000000 {
    device_type = "memory";
    reg = <0x0 0x80000000 0x0 0x800000>;
  };

  cpus {
    #address-cells = <0x00000001>;
    #size-cells = <0x00000000>;
    timebase-frequency = <0x2faf080>; // 50M
    cpu-map {
      cluster0 {
        core0 {
          cpu = <&core0>;
        };
      };
    };
    core0: cpu@0 {
      device_type = "cpu";
      reg = <0x00000000>;
      status = "okay";
      compatible = "riscv";
      riscv,isa = "rv64imacsu";
      mmu-type = "riscv,sv48";
    };
  };

  soc {
    #address-cells = <0x2>;
    #size-cells = <0x2>;

    PLIC: interrupt-controller@c000000 {
      #interrupt-cells = <0x00000001>;
      reg = <0x00000000 0x0c000000 0x00000000 0x04000000>;
      interrupt-controller;
      compatible = "riscv,plic0";
    };

    clint@2000000 {
      reg = <0x0 0x02000000 0x0 0x00010000>;
      compatible = "riscv;clint0";
    };
  };
};

```
