//! Platform-specific extensions to `std` for UEFI.

#![unstable(feature = "uefi_std", issue = "none")]

use crate::ffi::c_void;
use uefi_spec::efi::{Handle, SystemTable};
use uefi_spec::errors;
use uefi_spec::global_data::GlobalData;

static mut GLOBAL_SYSTEM_TABLE: GlobalData<SystemTable> = GlobalData::new();
static mut GLOBAL_SYSTEM_HANDLE: GlobalData<c_void> = GlobalData::new();

pub(crate) unsafe fn init_globals(
    handle: Handle,
    system_table: *mut SystemTable,
) -> Result<(), ()> {
    GLOBAL_SYSTEM_TABLE.init(system_table).map_err(|_| ())?;
    GLOBAL_SYSTEM_HANDLE.init(handle).map_err(|_| ())?;
    Ok(())
}

#[unstable(feature = "uefi_std", issue = "none")]
pub unsafe fn get_system_table() -> Result<&'static mut *mut SystemTable, errors::NullPtrError> {
    GLOBAL_SYSTEM_TABLE.get_mut()
}

#[unstable(feature = "uefi_std", issue = "none")]
pub unsafe fn get_system_handle() -> Result<&'static mut Handle, errors::NullPtrError> {
    GLOBAL_SYSTEM_HANDLE.get_mut()
}
