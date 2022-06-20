//! Platform-specific extensions to `std` for Unix platforms.
//!
//! Provides access to platform-level information on Unix platforms, and
//! exposes Unix-specific functions that would otherwise be inappropriate as
//! part of the core `std` library.
//!
//! It exposes more ways to deal with platform-specific strings ([`OsStr`],
//! [`OsString`]), allows to set permissions more granularly, extract low-level
//! file descriptors from files and sockets, and has platform-specific helpers
//! for spawning processes.
//!
//! [`OsStr`]: crate::ffi::OsStr
//! [`OsString`]: crate::ffi::OsString

#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
#[path = "../unsupported/args.rs"]
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
#[path = "../unsupported/fs.rs"]
pub mod fs;
#[path = "../unsupported/io.rs"]
pub mod io;
#[path = "../unsupported/locks/mod.rs"]
pub mod locks;
#[path = "../unsupported/net.rs"]
pub mod net;
#[path = "../unsupported/os.rs"]
pub mod os;
#[path = "../windows/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
#[path = "../unsupported/stdio.rs"]
pub mod stdio;
#[path = "../unsupported/thread.rs"]
pub mod thread;
#[path = "../unsupported/thread_local_key.rs"]
pub mod thread_local_key;
#[path = "../unsupported/time.rs"]
pub mod time;

#[path = "../unsupported/common.rs"]
#[deny(unsafe_op_in_unsafe_fn)]
mod common;
pub use common::*;

pub unsafe fn init(argc: isize, argv: *const *const u8) {
    use crate::os::uefi;
    use uefi_spec::efi;

    let args: &[*const u8] = unsafe { crate::slice::from_raw_parts(argv, argc as usize) };

    let handle: efi::Handle = args[0] as efi::Handle;
    let st: *mut efi::SystemTable = args[1] as *mut efi::SystemTable;

    unsafe { uefi::init_globals(handle, st).unwrap() };

    print_test(st);
}

fn print_test(st: *mut uefi_spec::efi::SystemTable) {
    let s = [0x0069u16, 0x006eu16, 0x0069u16, 0x0074u16, 0x000au16, 0x0000u16];
    unsafe {
        ((*(*st).con_out).output_string)((*st).con_out, s.as_ptr() as *mut uefi_spec::efi::Char16);
    }
}
