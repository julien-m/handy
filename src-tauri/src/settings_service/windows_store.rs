//! Windows settings files use checked handles, pinned parents and persistent user ACLs.
// @spec FR-003: Private Windows durable generations — .specs/features/008-settings-cli/spec.md#fr-003
#[path = "windows_store/replace.rs"]
mod replace;
#[path = "windows_store/security.rs"]
mod security;
pub(super) use replace::{discard_owned, replace_owned, ReplaceError};
use std::{
    ffi::OsStr,
    fs::File,
    io,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateDirectoryW, CreateFileW, FileIdInfo, GetFileInformationByHandle,
        GetFileInformationByHandleEx, GetFileType, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, DELETE,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_WRITE_THROUGH, FILE_ID_INFO, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TYPE_DISK, OPEN_EXISTING, READ_CONTROL,
    },
};

/// Expected resource type; reparse points and multiple file links are always refused.
#[derive(Clone, Copy)]
pub(crate) enum ResourceKind {
    File,
    Directory,
}
/// Stable identity uses the full Windows/ReFS file ID rather than a process-local handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FileIdentity {
    pub volume: u64,
    pub id: [u8; 16],
}
/// Ancestor handles deny deletion/rename until the caller's repository operation ends.
pub(crate) struct PinnedDirectory {
    handles: Vec<File>,
    path: PathBuf,
}
impl PinnedDirectory {
    /// Re-open the final pathname and compare it with the retained handle before mutation.
    pub(super) fn validate(&self) -> io::Result<()> {
        let current = open_raw(&self.path, ResourceKind::Directory, false, false)?;
        security::verify(&current, false)?;
        let held = self.handles.last().ok_or_else(security::denied)?;
        if identity(&current)? != identity(held)? {
            return Err(security::denied());
        }
        Ok(())
    }
    /// Canonical absolute parent path retained by this pinned directory.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}
/// Account SID for persisted-file ownership; IPC uses a separate logon identity.
pub(super) fn current_user_sid() -> io::Result<String> {
    security::current_user_sid()
}

/// Pin an existing directory and all ancestors without changing any existing ACL.
pub(super) fn pin_parent(path: &Path) -> io::Result<PinnedDirectory> {
    pin_directory(path, false)
}
/// Create only missing directory components with private attributes, then retain checked handles.
pub(super) fn ensure_private_parent(path: &Path) -> io::Result<PinnedDirectory> {
    pin_directory(path, true)
}

fn pin_directory(path: &Path, create: bool) -> io::Result<PinnedDirectory> {
    if !path.is_absolute() {
        return Err(security::denied());
    }
    let mut current = PathBuf::new();
    let mut handles = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                current.push(component);
                continue;
            }
            Component::RootDir | Component::Normal(_) => current.push(component),
            _ => return Err(security::denied()),
        }
        // Hold every already-checked ancestor before resolving the next component; reparse traversal is refused.
        let file = match open_raw(&current, ResourceKind::Directory, false, false) {
            Ok(file) => file,
            Err(error) if create && error.kind() == io::ErrorKind::NotFound => {
                create_directory(&current)?;
                let file = open_raw(&current, ResourceKind::Directory, false, false)?;
                security::verify(&file, true)?;
                file
            }
            Err(error) => return Err(error),
        };
        handles.push(file);
    }
    let final_handle = handles.last().ok_or_else(security::denied)?;
    // System ancestors have different owners; the settings parent must belong to this account.
    security::verify(final_handle, false)?;
    Ok(PinnedDirectory {
        handles,
        path: current,
    })
}

fn create_directory(path: &Path) -> io::Result<()> {
    let descriptor = security::Descriptor::private(&current_user_sid()?)?;
    let attributes = descriptor.attributes();
    let name = wide(path.as_os_str())?;
    // SAFETY: descriptor, attributes and terminated filename stay live until directory creation returns.
    if unsafe { CreateDirectoryW(name.as_ptr(), &attributes) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Open a legacy account-owned resource; only current-user/SYSTEM/Admin ACL grants are allowed.
pub(super) fn open_checked(path: &Path, kind: ResourceKind, writable: bool) -> io::Result<File> {
    let _parent = pin_parent(path.parent().ok_or_else(security::denied)?)?;
    let file = open_raw(path, kind, writable, !writable)?;
    security::verify(&file, false)?;
    Ok(file)
}
/// Open an existing private stable lock without granting replacement through share-delete.
pub(super) fn open_private(path: &Path, writable: bool) -> io::Result<File> {
    let _parent = pin_parent(path.parent().ok_or_else(security::denied)?)?;
    let file = open_raw(path, ResourceKind::File, writable, false)?;
    security::verify(&file, true)?;
    Ok(file)
}

fn open_raw(
    path: &Path,
    kind: ResourceKind,
    writable: bool,
    share_delete: bool,
) -> io::Result<File> {
    let name = wide(path.as_os_str())?;
    let access = GENERIC_READ | READ_CONTROL | if writable { GENERIC_WRITE } else { 0 };
    let share =
        FILE_SHARE_READ | FILE_SHARE_WRITE | if share_delete { FILE_SHARE_DELETE } else { 0 };
    let flags = FILE_FLAG_OPEN_REPARSE_POINT
        | if matches!(kind, ResourceKind::Directory) {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            0
        };
    // SAFETY: terminated filename remains live; no optional security/template pointers are dereferenced.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            access,
            share,
            std::ptr::null(),
            OPEN_EXISTING,
            flags,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful CreateFile transfers one owned handle; File closes it on all subsequent refusals.
    let file = unsafe { File::from_raw_handle(handle) };
    validate_kind(&file, kind)?;
    Ok(file)
}

/// Create an owner-private temp with DELETE rights. Existing paths are never truncated or ACL-repaired.
pub(super) fn create_private(path: &Path) -> io::Result<File> {
    create_file(path, DELETE)
}
/// Stable locks allow competing read/write handles while all holders deny pathname deletion.
pub(super) fn create_private_lock(path: &Path) -> io::Result<File> {
    create_file(path, 0)
}

fn create_file(path: &Path, delete_access: u32) -> io::Result<File> {
    let _parent = pin_parent(path.parent().ok_or_else(security::denied)?)?;
    let descriptor = security::Descriptor::private(&current_user_sid()?)?;
    let attributes = descriptor.attributes();
    let name = wide(path.as_os_str())?;
    // SAFETY: name/attributes/descriptor stay live; successful CREATE_NEW transfers sole handle ownership.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE | READ_CONTROL | delete_access,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &attributes,
            CREATE_NEW,
            FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_WRITE_THROUGH,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: valid owned handle is transferred exactly once, including validation errors.
    let file = unsafe { File::from_raw_handle(handle) };
    validate_kind(&file, ResourceKind::File)?;
    security::verify(&file, true)?;
    Ok(file)
}

fn validate_kind(file: &File, kind: ResourceKind) -> io::Result<()> {
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: File owns a live handle and initialized output has the API's exact structure size.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let directory = info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
    // SAFETY: handle remains live during this scalar metadata query.
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || directory != matches!(kind, ResourceKind::Directory)
        || (!directory && info.nNumberOfLinks != 1)
    {
        return Err(security::denied());
    }
    Ok(())
}

/// Query full volume/file identity; failed inspection never becomes equality or an unsafe fallback.
pub(super) fn identity(file: &File) -> io::Result<FileIdentity> {
    let mut info = FILE_ID_INFO::default();
    // SAFETY: live handle and aligned FILE_ID_INFO output match the declared byte size.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileIdInfo,
            (&mut info as *mut FILE_ID_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(FileIdentity {
        volume: info.VolumeSerialNumber,
        id: info.FileId.Identifier,
    })
}

fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut value: Vec<u16> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(security::denied());
    }
    value.push(0);
    Ok(value)
}
