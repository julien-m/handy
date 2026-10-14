//! Owned-handle replacement distinguishes refusal from uncertain post-commit durability.
use super::{security, validate_kind, wide, PinnedDirectory, ResourceKind};
use std::{ffi::OsStr, fs::File, io, os::windows::io::AsRawHandle};
use windows_sys::Win32::Storage::FileSystem::{
    FileDispositionInfo, FileRenameInfo, FlushFileBuffers, SetFileInformationByHandle,
    FILE_DISPOSITION_INFO, FILE_RENAME_INFO,
};

/// A failed flush after rename must not authorize an automatic retry of a possibly committed write.
#[derive(Debug)]
pub(crate) enum ReplaceError {
    Before(io::Error),
    After(io::Error),
}

/// Flush the temp, rename relative to a pinned parent, then report uncertain post-rename flush failures.
pub(crate) fn replace_owned(
    file: &File,
    parent: &PinnedDirectory,
    name: &OsStr,
) -> Result<(), ReplaceError> {
    let name = replacement_name(name).map_err(ReplaceError::Before)?;
    parent.validate().map_err(ReplaceError::Before)?;
    validate_kind(file, ResourceKind::File).map_err(ReplaceError::Before)?;
    security::verify(file, true).map_err(ReplaceError::Before)?;
    flush(file).map_err(ReplaceError::Before)?;
    let held = parent
        .handles
        .last()
        .ok_or_else(security::denied)
        .map_err(ReplaceError::Before)?;
    rename(file, held, &name).map_err(ReplaceError::Before)?;
    flush(file).map_err(ReplaceError::After)
}

/// Mark the held temp (opened with DELETE access) for deletion through its own handle, so a
/// substitute at the same pathname can never be removed; it disappears when the handle closes.
pub(crate) fn discard_owned(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: live owned handle; the initialized structure matches the declared byte size.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn replacement_name(name: &OsStr) -> io::Result<Vec<u16>> {
    let value = wide(name)?;
    let content = &value[..value.len() - 1];
    // A single normal filename excludes traversal, streams and Win32 trailing-dot/space aliases.
    if content.is_empty()
        || content.iter().any(|value| matches!(*value, 47 | 92 | 58))
        || matches!(content.last(), Some(32 | 46))
    {
        return Err(security::denied());
    }
    Ok(value)
}

fn rename(file: &File, parent: &File, name: &[u16]) -> io::Result<()> {
    let bytes = (name.len() - 1)
        .checked_mul(std::mem::size_of::<u16>())
        .ok_or_else(security::denied)?;
    let size = std::mem::offset_of!(FILE_RENAME_INFO, FileName)
        .checked_add(bytes)
        .ok_or_else(security::denied)?;
    let byte_count = u32::try_from(bytes).map_err(|_| security::denied())?;
    let native_size = u32::try_from(size).map_err(|_| security::denied())?;
    let mut buffer = vec![
        0usize;
        size.max(std::mem::size_of::<FILE_RENAME_INFO>())
            .div_ceil(std::mem::size_of::<usize>())
    ];
    // SAFETY: usize storage aligns the native structure and covers header/name; borrowed handles and UTF-16 stay live.
    unsafe {
        let rename = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
        (*rename).Anonymous.ReplaceIfExists = true;
        (*rename).RootDirectory = parent.as_raw_handle();
        (*rename).FileNameLength = byte_count;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*rename).FileName).cast(),
            name.len() - 1,
        );
        if SetFileInformationByHandle(
            file.as_raw_handle(),
            FileRenameInfo,
            rename.cast(),
            native_size,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn flush(file: &File) -> io::Result<()> {
    // SAFETY: File owns a live writable disk handle; this module never flushes a pipe or console.
    if unsafe { FlushFileBuffers(file.as_raw_handle()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
