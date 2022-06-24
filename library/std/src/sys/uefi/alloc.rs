use crate::alloc::{GlobalAlloc, Layout, System};
use crate::os::uefi;
use uefi_spec::{boot_services::memory_allocation_services, efi};

const POOL_ALIGNMENT: usize = 8;
const MEMORY_TYPE: u32 = efi::LOADER_DATA;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let st = unsafe {
            match uefi::get_system_table() {
                Ok(x) => x,
                Err(_) => return core::ptr::null_mut(),
            }
        };

        let align = layout.align();
        let size = layout.size();

        if size == 0 {
            return core::ptr::null_mut();
        }

        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let aligned_size = align_size(size, align);

        let r = memory_allocation_services::allocate_pool(st, MEMORY_TYPE, aligned_size, &mut ptr);

        if r.is_err() || ptr.is_null() {
            return core::ptr::null_mut();
        }

        unsafe { align_ptr(ptr.cast(), align) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let st = unsafe {
            match uefi::get_system_table() {
                Ok(x) => x,
                Err(_) => return,
            }
        };

        if layout.size() != 0 {
            let ptr = unsafe { unalign_ptr(ptr, layout.align()) };
            let r = memory_allocation_services::free_pool(st, ptr.cast());
            assert!(r.is_ok());
        }
    }
}

#[inline]
fn align_size(size: usize, align: usize) -> usize {
    if align > POOL_ALIGNMENT {
        // Allocate extra padding in order to be able to satisfy the alignment.
        size + align
    } else {
        size
    }
}

#[repr(C)]
struct Header(*mut u8);

#[inline]
unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
    if align > POOL_ALIGNMENT {
        let offset = align - (ptr.addr() & (align - 1));

        // SAFETY: `MIN_ALIGN` <= `offset` <= `layout.align()` and the size of the allocated
        // block is `layout.align() + layout.size()`. `aligned` will thus be a correctly aligned
        // pointer inside the allocated block with at least `layout.size()` bytes after it and at
        // least `MIN_ALIGN` bytes of padding before it.
        let aligned = unsafe { ptr.add(offset) };

        // SAFETY: Because the size and alignment of a header is <= `MIN_ALIGN` and `aligned`
        // is aligned to at least `MIN_ALIGN` and has at least `MIN_ALIGN` bytes of padding before
        // it, it is safe to write a header directly before it.
        unsafe { core::ptr::write((aligned as *mut Header).offset(-1), Header(ptr)) };

        aligned
    } else {
        ptr
    }
}

#[inline]
unsafe fn unalign_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
    if align > POOL_ALIGNMENT {
        // SAFETY: Because of the contract of `System`, `ptr` is guaranteed to be non-null
        // and have a header readable directly before it.
        unsafe { core::ptr::read((ptr as *mut Header).offset(-1)).0 }
    } else {
        ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align() {
        // UEFI ABI specifies that allocation alignment minimum is always 8. So this can be
        // statically verified.
        assert_eq!(POOL_ALIGNMENT, 8);

        // Loop over allocation-request sizes from 0-256 and alignments from 1-128, and verify
        // that in case of overalignment there is at least space for one additional pointer to
        // store in the allocation.
        for i in 0..256 {
            for j in &[1, 2, 4, 8, 16, 32, 64, 128] {
                if *j <= 8 {
                    assert_eq!(align_size(i, *j), i);
                } else {
                    assert!(align_size(i, *j) > i + std::mem::size_of::<*mut ()>());
                }
            }
        }
    }
}
