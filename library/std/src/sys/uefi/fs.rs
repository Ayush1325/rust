//! Implemented using File Protocol

use crate::ffi::OsString;
use crate::fmt;
use crate::hash::{Hash, Hasher};
use crate::io::{self, IoSlice, IoSliceMut, ReadBuf, SeekFrom};
use crate::os::uefi;
use crate::os::uefi::ffi::OsStrExt;
use crate::os::uefi::raw::protocols::file;
use crate::path::{Path, PathBuf};
use crate::ptr::NonNull;
use crate::sys::time::SystemTime;
use crate::sys::unsupported;

pub struct File {
    ptr: NonNull<uefi::raw::protocols::file::Protocol>,
}

#[derive(Clone)]
pub struct FileAttr {
    size: u64,
    perm: FilePermissions,
    file_type: FileType,
    created_time: SystemTime,
    last_accessed_time: SystemTime,
    modification_time: SystemTime,
}

pub struct ReadDir(!);

pub struct DirEntry(!);

#[derive(Clone, Debug)]
pub struct OpenOptions {
    open_mode: u64,
    attr: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FilePermissions {
    attr: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct FileType {
    attr: u64,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn perm(&self) -> FilePermissions {
        self.perm
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(self.modification_time)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        Ok(self.last_accessed_time)
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        Ok(self.created_time)
    }
}

impl From<file::Info> for FileAttr {
    fn from(info: file::Info) -> Self {
        FileAttr {
            size: info.file_size,
            perm: FilePermissions { attr: info.attribute },
            file_type: FileType { attr: info.attribute },
            modification_time: SystemTime::from(info.modification_time),
            last_accessed_time: SystemTime::from(info.last_access_time),
            created_time: SystemTime::from(info.create_time),
        }
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.attr & file::READ_ONLY != 0
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.attr |= file::READ_ONLY;
        } else {
            self.attr &= !file::READ_ONLY;
        }
    }
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.attr & file::DIRECTORY != 0
    }

    // Not sure if Archive is a file
    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }

    // Doesn't seem like symlink can be detected/supported.
    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.0
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0
    }

    pub fn file_name(&self) -> OsString {
        self.0
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        self.0
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        self.0
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        // These options open file in readonly mode
        OpenOptions { open_mode: 1, attr: 0 }
    }

    pub fn read(&mut self, read: bool) {
        if read {
            self.open_mode |= file::MODE_READ;
        } else {
            self.open_mode &= !file::MODE_READ;
        }
    }

    pub fn write(&mut self, write: bool) {
        if write {
            self.open_mode |= file::MODE_WRITE;
        } else {
            self.open_mode &= !file::MODE_WRITE;
        }
    }

    pub fn append(&mut self, _append: bool) {}

    pub fn truncate(&mut self, _truncate: bool) {}

    // Use const one upstream is fixed
    pub fn create(&mut self, create: bool) {
        if create {
            self.open_mode |= 0x8000000000000000;
        } else {
            self.open_mode &= !0x8000000000000000;
        }
    }

    pub fn create_new(&mut self, create_new: bool) {}
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        use crate::io::ErrorKind;
        use uefi::raw::Status;

        let rootfs = get_rootfs()?;
        let mut file_opened: *mut uefi::raw::protocols::file::Protocol = crate::ptr::null_mut();
        let r = unsafe {
            ((*rootfs.as_ptr()).open)(
                rootfs.as_ptr(),
                &mut file_opened,
                path.as_os_str().to_ffi_string().as_mut_ptr(),
                opts.open_mode,
                opts.attr,
            )
        };

        if r.is_error() {
            let e = match r {
                Status::NOT_FOUND => io::Error::new(
                    ErrorKind::NotFound,
                    "Specified file could not be found on the device",
                ),
                Status::WRITE_PROTECTED => io::Error::new(
                    ErrorKind::ReadOnlyFilesystem,
                    "An attempt was made to create a file, or open a file for write when
the media is write-protected.",
                ),
                Status::ACCESS_DENIED => {
                    io::Error::new(ErrorKind::PermissionDenied, "Service denied access to the file")
                }
                Status::VOLUME_FULL => io::Error::new(ErrorKind::StorageFull, "Volume Full"),
                Status::NO_MEDIA => io::Error::new(ErrorKind::Other, "Device has no medium"),
                Status::MEDIA_CHANGED => io::Error::new(
                    ErrorKind::Other,
                    "Device has a different medium in it or the medium is no longer
supported",
                ),
                Status::DEVICE_ERROR => {
                    io::Error::new(ErrorKind::Other, "Device reported an error")
                }
                Status::VOLUME_CORRUPTED => {
                    io::Error::new(ErrorKind::Other, "File system structures are corrupted")
                }
                _ => unreachable!(),
            };
            Err(e)
        } else {
            Ok(File {
                ptr: NonNull::new(file_opened)
                    .ok_or(io::Error::new(ErrorKind::Other, "File is Null"))?,
            })
        }
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        todo!()
    }

    pub fn fsync(&self) -> io::Result<()> {
        unimplemented!()
    }

    pub fn datasync(&self) -> io::Result<()> {
        unimplemented!()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unimplemented!()
    }

    pub fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }

    pub fn read_vectored(&self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        todo!()
    }

    pub fn is_read_vectored(&self) -> bool {
        todo!()
    }

    pub fn read_buf(&self, _buf: &mut ReadBuf<'_>) -> io::Result<()> {
        todo!()
    }

    pub fn write(&self, _buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    pub fn write_vectored(&self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        todo!()
    }

    pub fn is_write_vectored(&self) -> bool {
        todo!()
    }

    pub fn flush(&self) -> io::Result<()> {
        use uefi::raw::Status;

        let protocol = self.ptr;

        let r = unsafe { ((*protocol.as_ptr()).flush)(protocol.as_ptr()) };

        if r.is_error() {
            let e = match r {
                Status::WRITE_PROTECTED => io::Error::new(
                    io::ErrorKind::ReadOnlyFilesystem,
                    "The file or medium is write-protected.",
                ),
                Status::ACCESS_DENIED => io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "The file was opened read-only.",
                ),
                Status::VOLUME_FULL => {
                    io::Error::new(io::ErrorKind::StorageFull, "The volume is full.")
                }
                Status::VOLUME_CORRUPTED => io::Error::new(
                    io::ErrorKind::Other,
                    "The file system structures are corrupted.",
                ),
                Status::DEVICE_ERROR => {
                    io::Error::new(io::ErrorKind::Other, "The device reported an error.")
                }
                Status::NO_MEDIA => {
                    io::Error::new(io::ErrorKind::Other, "The device has no medium.")
                }
                _ => unreachable!(),
            };
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        todo!()
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unimplemented!()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unimplemented!()
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, _p: &Path) -> io::Result<()> {
        unsupported()
    }
}

impl fmt::Debug for File {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub fn readdir(_p: &Path) -> io::Result<ReadDir> {
    unsupported()
}

pub fn unlink(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    unsupported()
}

pub fn set_perm(_p: &Path, perm: FilePermissions) -> io::Result<()> {
    todo!()
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    unsupported()
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported()
}

pub fn try_exists(_path: &Path) -> io::Result<bool> {
    unsupported()
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(_p: &Path) -> io::Result<FileAttr> {
    unsupported()
}

pub fn lstat(_p: &Path) -> io::Result<FileAttr> {
    unsupported()
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    unsupported()
}

fn get_rootfs() -> io::Result<NonNull<uefi::raw::protocols::file::Protocol>> {
    use uefi::raw::protocols::{loaded_image, simple_file_system};

    let mut loaded_image_guid = loaded_image::PROTOCOL_GUID;
    let loaded_image_protocol =
        uefi::env::get_current_handle_protocol::<loaded_image::Protocol>(&mut loaded_image_guid)
            .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting Loaded Image Protocol"))?;

    let device_handle = unsafe { (*loaded_image_protocol.as_ptr()).device_handle };
    let device_handle = NonNull::new(device_handle)
        .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting Device Handle"))?;

    let mut simple_file_guid = simple_file_system::PROTOCOL_GUID;
    let simple_file_system_protocol =
        uefi::env::get_handle_protocol::<simple_file_system::Protocol>(
            device_handle,
            &mut simple_file_guid,
        )
        .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting Simple File System"))?;

    let mut file_protocol: *mut file::Protocol = crate::ptr::null_mut();
    let r = unsafe {
        ((*simple_file_system_protocol.as_ptr()).open_volume)(
            simple_file_system_protocol.as_ptr(),
            &mut file_protocol,
        )
    };
    if r.is_error() {
        Err(io::Error::new(io::ErrorKind::Other, "Error getting rootfs"))
    } else {
        NonNull::new(file_protocol)
            .ok_or(io::Error::new(io::ErrorKind::Other, "Error getting rootfs"))
    }
}
