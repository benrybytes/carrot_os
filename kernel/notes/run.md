# making rust not use standard libraries

operating system should not contain any standard libraries, as they
are dependent on the operating system they are running on

we could stop using any standard libraries in rust with the
following attribute:

```rs
// main.rs

#![no_std]

fn main() {
    println!("Hello, world!");
}
```

# default language items on the compiler should not be used

**language items** special functions and types required by the compiler to know, such as
copying an item requires `#[lang = "copy"]`

**stack unwinding** process in calling functions line by line, and calling **destructors**
of local variables even after an error is thrown

-   **eh personality** an example of language item that unwinds the function to free memory and
    catch panics while continuing executions

if we want to stop executions instead of going, we can disable it

```rs
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

it will give an error indicating a start method is needed as main is not the first thing called

# overwriting C runtime zero entry point

```sh
using main requires the standard library
```

**start language item** used by rust as `C runtime zero`, to implement stack overflow guards
or printing backtraces on panics

we do not want this in our runtime, as our program does not rely on standard library

we could overwrite the `C runtime zero` using an attribute to not run the **normal entry point chain**

```rs
#![no_std]
#![no_main]

...
```

alongside, we want our **linker** to look at another entry point instead of the `C runtime zero` entry point

```rs
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

# running without an operating system default CC linkers

**ABI** how data structures or anything of the code is read by the machine code

running on bare metal means to run on device by using flags targeting no OS

```sh
rustup target add thumbv7em-none-eabihf # run on an ARM based architecture targeting no operating system with none
cargo build --target thumbv7em-none-eabihf
```
