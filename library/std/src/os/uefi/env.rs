//! UEFI-specific extensions to the primitives in `std::env` module

use super::raw::{BootServices, Handle, SystemTable};
use crate::ffi::c_void;
use crate::ptr::NonNull;
use crate::sync::atomic::{AtomicPtr, Ordering};

static GLOBAL_SYSTEM_TABLE: GlobalData<SystemTable> = GlobalData::new();
static GLOBAL_SYSTEM_HANDLE: GlobalData<c_void> = GlobalData::new();

#[unstable(feature = "uefi_std", issue = "none")]
/// Initializes Global Atomic Pointers to SystemTable and Handle.
/// Should only be called once in the program execution
/// Returns error if the initialization fails. This can happen if the pointers have already been
/// initialized before or if either of the supplied pointer is null. The std becomes almost
/// useless if this fails.
/// FIXME: Improve Erorr
pub unsafe fn init_globals(handle: Handle, system_table: *mut SystemTable) -> Result<(), ()> {
    GLOBAL_SYSTEM_TABLE.init(system_table).map_err(|_| ())?;
    GLOBAL_SYSTEM_HANDLE.init(handle).map_err(|_| ())?;
    Ok(())
}

#[unstable(feature = "uefi_std", issue = "none")]
/// This function returns error if SystemTable pointer is null
/// On success, the returned pointer is guarenteed to be non-null.
pub fn get_system_table() -> Option<NonNull<SystemTable>> {
    GLOBAL_SYSTEM_TABLE.load()
}

#[unstable(feature = "uefi_std", issue = "none")]
/// This function returns error if SystemHandle pointer is null
/// On success, the returned pointer is guarenteed to be non-null.
pub fn get_system_handle() -> Option<NonNull<c_void>> {
    GLOBAL_SYSTEM_HANDLE.load()
}

#[unstable(feature = "uefi_std", issue = "none")]
/// On success, the returned pointer is guarenteed to be non-null.
/// SAFETY: Do not cache the returned pointer unless you are sure it will remain valid.
pub fn get_boot_services() -> Option<NonNull<BootServices>> {
    let system_table = get_system_table()?;
    let boot_services = unsafe { (*system_table.as_ptr()).boot_services };
    NonNull::new(boot_services)
}

/// It is mostly ment to store Global pointers
struct GlobalData<T> {
    ptr: AtomicPtr<T>,
}

impl<T> GlobalData<T> {
    /// Initializes GlobalData with internal NULL pointer. This is constant so that it can be used
    /// in statics.
    const fn new() -> Self {
        Self { ptr: AtomicPtr::new(core::ptr::null_mut()) }
    }

    /// SAFETY: This function will only initialize once.
    /// The return value is a Result containing nothing if it is success. In the case of an
    /// error, it returns the previous pointer.
    fn init(&self, ptr: *mut T) -> Result<(), *mut T> {
        // Check that the ptr is not null.
        if ptr.is_null() {
            return Err(ptr);
        }

        let r = self.ptr.compare_exchange(
            core::ptr::null_mut(),
            ptr,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );

        match r {
            Ok(_) => Ok(()),
            Err(x) => Err(x),
        }
    }

    /// The return value is a non-null pointer.
    fn load(&self) -> Option<NonNull<T>> {
        let p = self.ptr.load(Ordering::Relaxed);
        NonNull::new(p)
    }
}
