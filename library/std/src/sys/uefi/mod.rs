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
pub mod net;
pub mod os;
#[path = "../windows/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
pub mod stdio;
#[path = "../unsupported/thread.rs"]
pub mod thread;
#[path = "../unsupported/thread_local_key.rs"]
pub mod thread_local_key;
#[path = "../unsupported/time.rs"]
pub mod time;

use crate::io as std_io;
use crate::os::uefi;
use uefi_spec::efi;

pub mod memchr {
    pub use core::slice::memchr::{memchr, memrchr};
}

pub unsafe fn init(argc: isize, argv: *const *const u8) {
    let args: &[*const u8] = unsafe { crate::slice::from_raw_parts(argv, argc as usize) };

    let handle: efi::Handle = args[0] as efi::Handle;
    let st: *mut efi::SystemTable = args[1] as *mut efi::SystemTable;

    unsafe { uefi::init_globals(handle, st).unwrap() };

    print_test();
    // println!("abc");
}

fn print_test() {
    use crate::io::Write;
    use crate::{string::String, vec::Vec};
    use uefi_spec::protocols::simple_text_output;

    let _ = stdio::Stdout::new().write("init\n".as_bytes());
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

pub fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> std_io::Error {
    std_io::const_io_error!(
        std_io::ErrorKind::Unsupported,
        "operation not supported on this platform",
    )
}

pub fn decode_error_kind(_code: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Uncategorized
}

pub fn abort_internal() -> ! {
    use uefi_spec::boot_services::image_services;

    if let (Ok(st), Ok(handle)) =
        (unsafe { uefi::get_system_table() }, unsafe { uefi::get_system_handle() })
    {
        let _ = image_services::exit(st, handle, efi::Status::ABORTED, &mut [0]);
    }

    core::intrinsics::abort();
}

pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}
