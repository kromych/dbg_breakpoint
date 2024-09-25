mod dbg;
mod dbg_win_seh;

/// Execute the breakpoint instruction. That might crash the program if the debugger/tracer
/// is not able to step over the instruction.
#[macro_export]
macro_rules! breakpoint {
    () => {
        // SAFETY: Executing the breakpoint instruction. No state is shared
        // or modified by this code.
        unsafe {
            #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
            core::arch::asm!("int3");

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("brk #0xf000");

            #[cfg(target_arch = "arm")]
            core::arch::asm!("udf #254");

            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            core::arch::asm!("ebreak");

            #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
            core::arch::asm!("trap");
        }

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_arch = "powerpc",
            target_arch = "powerpc64"
        )))]
        compile_error!("Debug trap instruction is not supported on this architecture.");
    };
}

/// Presence of a debugger/tracer. The debugger being concerned
/// is expected to use the OS API to debug this process.
#[derive(Copy, Clone, Debug)]
#[allow(unused)]
pub enum DebuggerPresence {
    /// The debugger is attached to this process.
    Detected,
    /// The debugger is not attached to this process.
    NotDetected,
}

pub use dbg::breakpoint_if_debugging;
pub use dbg::is_debugger_present;

// Don't have a 32-bit Windows around, might try a VM.
#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
pub use dbg_win_seh::breakpoint_if_debugging_seh;
