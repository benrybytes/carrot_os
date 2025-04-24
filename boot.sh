#/bin/bash
cargo +nightly build
cargo +nightly bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64_operating_system/debug/bootimage-operating_system.bin
