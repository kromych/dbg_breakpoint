//! Debugging aids on Windows employing SEH to detect the debugger if it is
//! hiding its presence TEB manipulation.
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
//!
//! To learn more about SEH attributes empirically, can compile C files to
//! assembly with `clang` or `cl`. As for the documentation, here are some
//! links:
//!
//! * https://doc.rust-lang.org/reference/inline-assembly.html#structured-exception-handling
//! * https://learn.microsoft.com/en-us/cpp/build/arm64-exception-handling?view=msvc-170
//! * https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64?view=msvc-170

#![cfg(target_os = "windows")]

use crate::DebuggerPresence;

#[cfg(not(target_arch = "x86"))]
extern "C" {
    /// Breakpoint that is passed to the debugger as the first chance exception
    /// if the debugger is attached, and is skipped over otherwise.
    /// Returns `0` if no debugger was sensed, and `-1` if it was.
    fn __dbg_breakpoint() -> i32;
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
        mov             eax, -1
    2:
    3:
        add             rsp, 64
        ret
    4:
        xor             eax, eax
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
        mov             w0, #1     // EXCEPTION_EXECUTE_HANDLER
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
        mov             w0, #-1
    2:
    3:
        .seh_startepilogue
        ldr             lr, [sp], #16
        .seh_save_reg_x lr, 16
        .seh_endepilogue
        ret
    4:
        mov             w0, wzr
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

#[cfg(not(target_arch = "x86"))]
pub fn breakpoint_if_debugging_seh() -> Option<DebuggerPresence> {
    // SAFETY: the call does not access any state shared between threads.
    match unsafe { __dbg_breakpoint() } {
        0 => Some(DebuggerPresence::NotDetected),
        -1 => Some(DebuggerPresence::Detected),
        _ => panic!("Internal error"),
    }
}
