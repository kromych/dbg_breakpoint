//! Debugging aids.
//!
//! On Windows, the approach is based on the Structured Exception Handling:
//! https://learn.microsoft.com/en-us/windows/win32/debug/structured-exception-handling
//! and is conceptually equivalent to
//!
//! ```c
//! __try {
//!     __debugbreak();
//! } __except (1) {
//!     /* Nothing */
//! }
//! ```
//!
//! The implementation is available for the `x86_64` and the `aarch64`
//! targets under Windows.

#[cfg(target_os = "windows")]
mod windows {
    #[cfg(not(target_arch = "x86"))]
    extern "C" {
        /// Breakpoint that is passed to the debugger as the first chance exception
        /// if the debugger is attached, and is skipped over otherwise.
        pub(super) fn __dbg_breakpoint();
    }

    #[cfg(target_arch = "x86_64")]
    core::arch::global_asm!(
        r#"
        .pushsection    .text

        .globl          __dbg_breakpoint_flt
        .p2align        4
    __dbg_breakpoint_flt:
        mov             eax, 1     # EXCEPTION_EXECUTE_HANDLER
        ret

        .globl          __dbg_breakpoint
        .p2align        4
        .def            __dbg_breakpoint; .scl 2; .type 32; .endef
        .seh_proc       __dbg_breakpoint
    __dbg_breakpoint:
        sub             rsp, 64
        .seh_stackalloc 64
        .seh_endprologue
    1:
        int3
    2:
    3:
        add             rsp, 64
        ret
    4:
        jmp             3b
        .seh_handler    __C_specific_handler, @except
        .seh_handlerdata
        .long 1                             # One handler entry
        .long (1b)@IMGREL                   # Start address of __try block
        .long (2b)@IMGREL                   # End address of __try block
        .long (__dbg_breakpoint_flt)@IMGREL # Exception filter
        .long (4b)@IMGREL                   # Exception handler
        .text
        .seh_endproc
        .popsection
    "#
    );

    #[cfg(target_arch = "aarch64")]
    core::arch::global_asm!(
        r#"
        .pushsection    .text

        .globl          __dbg_breakpoint_flt
        .p2align        2
    __dbg_breakpoint_flt:
        mov             w0, 1     // EXCEPTION_EXECUTE_HANDLER
        ret

        .globl          __dbg_breakpoint
        .p2align        2
        .def            __dbg_breakpoint; .scl 2; .type 32; .endef
        .seh_proc       __dbg_breakpoint
    __dbg_breakpoint:
        str             lr, [sp, #-16]!
        .seh_save_reg_x lr, 16
        .seh_endprologue
    1:
        brk             #0xf000
    2:
    3:
        .seh_startepilogue
        ldr             lr, [sp], #16
        .seh_save_reg_x lr, 16
        .seh_endepilogue
        ret
    4:
        b               3b
        .seh_handler    __C_specific_handler, @except
        .seh_handlerdata
        .long 1                             // One handler entry
        .long (1b)@IMGREL                   // Start address of __try block
        .long (2b)@IMGREL                   // End address of __try block
        .long (__dbg_breakpoint_flt)@IMGREL // Exception filter
        .long (4b)@IMGREL                   // Exception handler
        .text
        .seh_endproc
        .popsection
    "#
    );

    #[cfg(target_arch = "x86")]
    pub(super) fn __dbg_breakpoint() {
        // Not implemented
    }
}

/// Breakpoint that is passed to the debugger as the first chance exception
/// if the debugger is attached, and is skipped over otherwise.
#[cfg(target_os = "windows")]
pub fn dbg_breakpoint() {
    // SAFETY: the call does not access any state shared between threads.
    unsafe {
        windows::__dbg_breakpoint();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn dbg_breakpoint() {
    // Not implemented
}
