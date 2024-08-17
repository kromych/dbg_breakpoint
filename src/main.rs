#![feature(core_intrinsics)]

mod dbg;

#[cfg(target_os = "windows")]
mod dbg_win_seh;

fn main() {
    #[cfg(target_os = "windows")]
    dbg_win_seh::dbg_breakpoint();

    dbg::breakpoint_if_debugging();

    println!("Hello, world!");
}
