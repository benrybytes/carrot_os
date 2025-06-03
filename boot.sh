#/bin/bash

# steps:
# make sure to not include "bootimage" crate in Cargo.toml
# ensure to install all rustup features
cargo +nightly build
cargo +nightly bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64_operating_system/debug/bootimage-carrot_os.bin
