#/bin/bash

# steps:
# make sure to not include "bootimage" crate in Cargo.toml
# rustup +nightly-2024-12-07-aarch64-apple-darwin | tested version that worked with no dependency issues
# ensure to install all rustup features
# cargo +nightly-2024-12-07-aarch64-apple-darwin build
# cargo +nightly-2024-12-07-aarch64-apple-darwin bootimage
# qemu-system-x86_64 -drive format=raw,file=target/x86_64_carrot_os/debug/bootimage-carrot_os.bin

# using limine
make clean && make run
