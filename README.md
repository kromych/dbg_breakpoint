# Breakpoints when the debugger is attached

Set breakpoints with the `breakpoint!()` macro on many target architectures
and popular OSes like FreeBSD, macOS, iOS, Linux distro's, Windows. Break into
the debugger with an easy `breakpoint_if_debugging()` call!

Well, sure, but why?

* It might be more convinient to add the call to `breakpoint_if_debugging` from inside
  the comfort of your editor than to remember the incantion in the debugger,
* Some callsites like lambdas and async routines/coroutines can be tricky to set a
  breakpoint to in the debugger due to name mangling or because the toolchain doesn't
  give them a name that is easily-discovered/human-friendly,
* Can add this to your `#[panic_handler]` to break into the debugger on a panic.

Here is the example of how one can make use of this: [`runme.rs`](src/bin/runme.rs).
Do exercise *extreme* caution when using any of this in the production environment, i.e.
out of the inner development loop. Heisenbugs and crashes might be sighted.

Platform- and target-specific notes follow.

## Windows

The library provides `breakpoint_if_debugging()` and `breakpoint_if_debugging_seh()`
The latter might be useful to detect the debugger if it is trying to hide its presence
via some cheap tricks.

## Linux, macOS and FreeBSD

The debugger detection logic will detect any tracer like `strace` as the debugger, and
if the tracer isn't able to skip over the breakpoint CPU instruction, the program will
crash. That can be fixed by handling `SIGTRAP` inside your program.

## arm64

`brk #imm16` is used for breakpoint on arm64.

Just FYI, the `#imm16` value can be inside the Linux kernel 6.1
at the time of writing:

* `0x004`: for installing kprobes
* `0x005`: for installing uprobes
* `0x006`: for kprobe software single-step
* `0x400` - `0x7ff`: kgdb
* `0x100`: for triggering a fault on purpose (reserved)
* `0x400`: for dynamic BRK instruction
* `0x401`: for compile time BRK instruction
* `0x800`: kernel-mode BUG() and WARN() traps
* `0x9xx`: tag-based KASAN trap (allowed values 0x900 - 0x9ff)
* `0x8xxx`: Control-Flow Integrity traps

Here, we're talking the user mode yet the above illustrates the point
that the value supplied after `brk` influences what to expect.

For `__builtin_trap()`, `gcc` produces `brk #0x3e8`, `clang` generates `brk #1`.
This library uses `0xf000` as the debuggers skip over the debug trap automatically
in this case.
