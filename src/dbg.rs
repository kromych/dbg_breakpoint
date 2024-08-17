//! Debugging aids.

/// Presence of a debugger.
#[derive(Copy, Clone, Debug)]
pub(crate) enum DebuggerPresence {
    /// The debugger is attached to the process.
    Detected,
    /// The debugger is not attached to the process.
    NotDetected,
}

/// Error detecting debugger presence.
#[derive(Copy, Clone, Debug)]
pub(crate) enum DebuggerPresenceError {
    /// The functionality is not available.
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "freebsd",)))]
    NotImplemented,
    /// The detection logic failed to determine
    /// the debugger presence. It may or may not be
    /// attached.
    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "linux"))]
    DetectionFailed,
}

#[cfg(target_os = "windows")]
mod os {
    use super::{DebuggerPresence, DebuggerPresenceError};

    #[link(name = "kernel32")]
    extern "system" {
        fn IsDebuggerPresent() -> i32;
    }

    pub(super) fn is_debugger_present() -> Result<DebuggerPresence, DebuggerPresenceError> {
        // SAFETY: No state is shared between threads. The call reads
        // a field from the Thread Environment Block using the OS API
        // as required by the documentation.
        if unsafe { IsDebuggerPresent() } != 0 {
            Ok(DebuggerPresence::Detected)
        } else {
            Ok(DebuggerPresence::NotDetected)
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
mod os {
    use super::{DebuggerPresence, DebuggerPresenceError};
    use libc::{c_int, c_void, sysctl, CTL_KERN, KERN_PROC, KERN_PROC_PID};
    use std::{mem::size_of_val, process};

    #[cfg(target_os = "macos")]
    mod traced {
        const P_TRACED: i32 = 0x00000800;

        // The structure definition contain nested structure and plethora
        // of fields that are not interesting here. Combine all the unnecessary
        // fields into one array. `libc` doesn't appear to have the definition
        // at the moment.
        #[repr(C)]
        pub(super) struct KinfoProc {
            _unused0: [u8; 32],
            p_flag: i32,
            _unused1: [u8; 612],
        }

        impl KinfoProc {
            pub(super) fn is_traced(&self) -> bool {
                println!("{:x}", self.p_flag);
                (self.p_flag & P_TRACED) != 0
            }
        }
    }

    #[cfg(target_os = "freebsd")]
    mod traced {
        const P_TRACED: i32 = 0x00000002;

        #[repr(C)]
        pub(super) struct KinfoProc {
            _ki_structsize: i32, // Size of the structure
            ki_flag: i32,        // Process flags (important for P_TRACED)
            _ki_pid: i32,        // Process ID (useful for identification)
            _ki_ppid: i32,       // Parent process ID
            _ki_tid: i32,        // Thread ID
            _ki_traced: u8,      // Tracing status (indicates if being traced)
            _unused: [u8; 496],  // Combine all unnecessary fields here
        }

        impl KinfoProc {
            pub(super) fn is_traced(&self) -> bool {
                (self.ki_flag & P_TRACED) != 0
            }
        }
    }

    pub(super) fn is_debugger_present() -> Result<DebuggerPresence, DebuggerPresenceError> {
        let mut info = unsafe { std::mem::zeroed::<traced::KinfoProc>() };
        let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PID, process::id() as c_int];
        let mut info_size = size_of_val(&info);

        // SAFETY: No state is shared with other threads. The sysctl call
        // is safe according to the documentation.
        if unsafe {
            sysctl(
                mib.as_mut_ptr(),
                mib.len() as u32,
                &mut info as *mut _ as *mut c_void,
                &mut info_size,
                std::ptr::null_mut(),
                0,
            )
        } == 0
        {
            if info.is_traced() {
                Ok(DebuggerPresence::Detected)
            } else {
                Ok(DebuggerPresence::NotDetected)
            }
        } else {
            Err(DebuggerPresenceError::DetectionFailed)
        }
    }
}

#[cfg(target_os = "linux")]
mod os {
    use super::{DebuggerPresence, DebuggerPresenceError};
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    pub(super) fn is_debugger_present() -> Result<DebuggerPresence, DebuggerPresenceError> {
        let file =
            File::open("/proc/self/status").map_err(|_| DebuggerPresenceError::NotImplemented)?;
        let reader = BufReader::new(file);

        for line in reader.lines().flatten() {
            if line.starts_with("TracerPid:") {
                return if line
                    .split(':')
                    .nth(1)
                    .map_or(false, |pid| pid.trim() != "0")
                {
                    Ok(DebuggerPresence::Detected)
                } else {
                    Ok(DebuggerPresence::NotDetected)
                };
            }
        }

        Err(DebuggerPresenceError::DetectionFailed)
    }
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "linux"
)))]
mod os {
    use super::{DebuggerPresence, DebuggerPresenceError};

    pub(super) fn is_debugger_present() -> Result<DebuggerPresence, DebuggerPresenceError> {
        Err(DebuggerPresenceError::NotImplemented)
    }
}

/// Detect the debugger presence.
pub(crate) fn is_debugger_present() -> Result<DebuggerPresence, DebuggerPresenceError> {
    os::is_debugger_present()
}

/// Execute the breakpoint instruction if the debugger presence is detected.
/// This is racy and does not try to detect the debugger at all costs (e.g.,
/// when anti-debugger tricks are at play). Useful for breaking into the
/// debugger without the need to set a breakpoint in the debugger.
pub(crate) fn breakpoint_if_debugging() {
    if let Ok(DebuggerPresence::Detected) = is_debugger_present() {
        // SAFETY: Executing the breakpoint instruction. No state is shared
        // or modified by this code.
        unsafe {
            std::intrinsics::breakpoint();
        }
    }
}
