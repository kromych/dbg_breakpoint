use dbg_breakpoint::breakpoint;
use dbg_breakpoint::is_debugger_present;

fn main() {
    let is_debugger_present = is_debugger_present();
    println!("Is debugger present: {is_debugger_present:?}");

    #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
    {
        let is_debugger_present = dbg_breakpoint::breakpoint_if_debugging_seh();
        println!("Windows 64-bit SEH: is debugger present: {is_debugger_present:?}");
    }

    println!("Now the process will crash if the debugger is not attcahed");
    breakpoint!();
}
