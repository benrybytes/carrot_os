# boot

boot, such as BIOS (Basic input / output system) allows to execute
firmware code from the motherboard's ROM (read on memory) to initialize CPU, read
available RAM, etc.

Then, it starts to boot after finding the bootable disk

# BIOS boot

an old bootloader to run firmware checkers

determines location of kernel and switches the CPU from 16, to 32, and
finally 64 bit mode

# using rust nightly

be able to test unofficial released features

```sh
rustup override set nightly
```

# minimal kernel

will not be ran through cargo, as it builds for our host system, so
we need to define our target

here, we create a json file to specify a basic target system, in this case we
are targeting x86_64 systems

```json
{
	"llvm-target": "x86_64-unknown-none",
	"data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
	"arch": "x86_64",
	"target-endian": "little",
	"target-pointer-width": "64",
	"target-c-int-width": "32",
	"os": "none",
	"executables": true
}
```

we want to use Rust's linker instead of gcc which can sometimes not be available
for some systems of Linux

```json
"linker-flavor": "ld.lld",
"linker": "rust-lld",
```

making sure our target too does not accept stack unwinding as our toml file

```json
"panic-strategy": "abort",
```

stopping the redzone is important for our OS to handle exceptions, rather than
corruption because this process is done on the kernel, and not the OS, and could
cause conflict in our host and custom OS

essentially, it calls another 128-byte stack for exceptions, but cannot handle together
with our OS and userspace at the same time, so we want to make OS independent
from host exception handling

```json
"disable-redzone": true,
```

# disabling SIMD

otherwise known as **Single Instruction Multiple Data\***, it is a process
for floating point operations, in allocating huge amount of registers beforehand

Alongside, forcing the kernel to restore SIMD if at any point it panics, causing more
wait times

instead, we disable both `mmx` and `sse` features used in SIMD, and enable `soft-float`
to make integers emulate floats if needed

```json
"features": "-mmx,-sse,+soft-float",
```

# final file

```json
{
	"llvm-target": "x86_64-unknown-none",
	"data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
	"arch": "x86_64",
	"target-endian": "little",
	"target-pointer-width": "64",
	"target-c-int-width": "32",
	"os": "none",
	"executables": true,
	"linker-flavor": "ld.lld",
	"linker": "rust-lld",
	"panic-strategy": "abort",
	"disable-redzone": true,
	"features": "-mmx,-sse,+soft-float"
}
```

run it with the target flag

```sh
cargo run --target my-operating-system.json
```

# error with core

it will give an exception of core not found, the library holding default interfaces as
`Option`, `Result`, etc

reason is, this crate is a **precompiled library**, meaning it is already made in our OS, but
not in our target OS, so we need to recompile it again

in order to recompile, we could specify in the `.cargo/config.toml`

```toml
[unstable]
build-std = ["core", "compiler_builtins"]
```

these crates libraries require us to use rust's source code to access the code
we need to recompile

this component allows us to include rust's standard library source code 
```sh
rustup component add rust-src
```

we want to use nightly to compile and use the unstable.build-std feature from nightly

```sh
cargo +nightly run --target x86_64_operating_system.json
```

# prevent using of C based memory packages

we can instead use rust's memory crate to copy memory and just integrate
ways to safely use memory based functions

```toml
# .cargo/config.toml
[unstable]
build-std-features = ["compiler-builtins-mem"]
build-std = ["core", "compiler_builtins"]
```

we can also set our build or run argument to use our json file instead

```toml
[build]
target = "x86_64_operating_system.json"

[run]
target = "x86_64_operating_system.json"

```

# printing text using VGA text buffer

```rs
// converting each character to a UTF-8 byte as that is what rust uses
static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8; // unsafe mutable pointer at default VGA address

    for (i, &byte) in HELLO.iter().enumerate() { // iterate and reference character slice

        // unsafe: we are changing the pointer to be of this value
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;

            // using pointer arithmitic to set color of text
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // add a + 1 bit next to our character for color
        }
    }

    loop {}
}

```

# creating a boot image

using the bootloader dependency can allow us to make an image of our
OS

```toml
# Cargo.toml
...
[dependencies]
bootloader = "0.9"
```

alongside, install the cargo package in your host machine, go to `~` directory


```sh
cargo install bootimage
```

as well as, being able to have the llvm component to allow building and previewing
the image in a virtual machine

```sh
# aarch64 to be able to run qemu from aarch64 to llvm
rustup component add rust-src --toolchain nightly-aarch64-apple-darwin
rustup component add llvm-tools-preview
```
make sure to first build your cargo image, then you run

```sh
cargo +nightly build
cargo update -p bootloader # make sure to do this important step to be able to use bootimage
cargo bootimage
```

## using qemu after making the image

after creating the image in your build directory, you can
run it through qemu

if using arch linux, you have to install the full
qemu package for it to work

```sh
doas pacman -Sy qemu-full
```

afterwards, you can run it and a window should be displayed

```sh
qemu-system-x86_64 -drive format=raw,file=target/x86_64_operating_system/debug/bootimage-operating_system.bin
```

## running bootimage with qemu

bootimage crate has a key called `runner`, in order to read the bin of the made image

by targetting all files with no OS with the target os argument in configuration macro.
Afterwards, we run the bootimage runner passing the image automatically from the build

```toml
# allows run to execute this as well
[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```
