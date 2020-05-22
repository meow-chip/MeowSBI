#!/bin/bash

qemu-system-riscv64 \
  -m 2G \
  -nographic \
  -machine virt \
  -kernel ./target/riscv64gc-unknown-none-elf/debug/meow-sbi
  # -append "root=/dev/vda ro" \
  # -object rng-random,filename=/dev/urandom,id=rng0 \
  # -device virtio-rng-device,rng=rng0 \
  # -device virtio-blk-device,drive=hd0 \
  # -drive file=./2020.raw,format=raw,id=hd0 \
  # -device virtio-net-device,netdev=usernet \
  # -netdev user,id=usernet,hostfwd=tcp::10000-:22 \
  # -device loader,file=./2020.payload.elf
  # -device loader,file=../u-boot/u-boot
