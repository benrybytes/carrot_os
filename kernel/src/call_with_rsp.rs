use core::arch::naked_asm;

/// # Safety
/// Stack must be valid
#[unsafe(naked)]

// this goes based on calling conventions of sysv64, where first parameter holds rdi being our new
// stack, second holds rsi, being the function pointer
pub unsafe extern "sysv64" fn call_with_rsp(new_rsp: u64, f: extern "sysv64" fn() -> !) -> ! {
    naked_asm!(
        "
        mov rsp, rdi
        call rsi
        "
    );
}
