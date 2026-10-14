//! Current-user file ownership and private creation ACLs, independent of logon-session IPC.
use std::{fs::File, io, os::windows::io::AsRawHandle};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree},
    Security::{
        AclSizeInformation,
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
        },
        GetAce, GetAclInformation, GetSecurityDescriptorControl, GetTokenInformation, IsValidSid,
        TokenUser, ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, ACL_SIZE_INFORMATION,
        DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PSID, SECURITY_ATTRIBUTES,
        SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};
// Token inspection is bounded like the settings IPC implementation; SID text is bounded by Windows SID limits.
const TOKEN_BYTES: usize = 65_536;
const SID_CHARS: usize = 192;
// ACCESS_ALLOWED_ACE_TYPE from winnt.h; the bindings place this constant in an otherwise unused SDK feature.
const ALLOW_ACE: u8 = 0;

pub(super) fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "private settings resource unavailable",
    )
}

/// LocalAlloc descriptor ownership; its borrowed SID/ACL pointers expire on drop.
pub(super) struct Descriptor(*mut core::ffi::c_void);
impl Drop for Descriptor {
    fn drop(&mut self) {
        // SAFETY: construction owns exactly one LocalAlloc descriptor; drop frees it once.
        unsafe {
            LocalFree(self.0);
        }
    }
}
impl Descriptor {
    /// Create a protected, non-inheritable descriptor granting only the persistent account SID.
    pub(super) fn private(sid: &str) -> io::Result<Self> {
        // Explicit owner avoids the Administrators default owner of an elevated token.
        let sddl: Vec<u16> = format!("O:{sid}D:P(A;;FA;;;{sid})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut descriptor = std::ptr::null_mut();
        // SAFETY: terminated UTF-16 and initialized descriptor output live through the conversion.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }
    /// Borrow non-inheritable native creation attributes; this descriptor must outlive the API call.
    pub(super) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        }
    }
}

/// Return TokenUser identity, never the temporary TokenLogonSid used by IPC.
pub(super) fn current_user_sid() -> io::Result<String> {
    let mut token = std::ptr::null_mut();
    // SAFETY: process pseudo-handle and initialized token output are valid.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = token_user(token);
    // SAFETY: successful OpenProcessToken transfers this owned handle, closed once after inspection.
    unsafe {
        CloseHandle(token);
    }
    result
}

fn token_user(token: windows_sys::Win32::Foundation::HANDLE) -> io::Result<String> {
    let mut buffer = vec![0usize; TOKEN_BYTES / std::mem::size_of::<usize>()];
    let mut length = 0;
    // SAFETY: usize allocation aligns TOKEN_USER; bounded writable storage/token remain live throughout.
    unsafe {
        if GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            TOKEN_BYTES as u32,
            &mut length,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        if length < std::mem::size_of::<TOKEN_USER>() as u32 || length as usize > TOKEN_BYTES {
            return Err(denied());
        }
        sid_text((*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid)
    }
}

fn sid_text(sid: PSID) -> io::Result<String> {
    let mut text = std::ptr::null_mut();
    // SAFETY: caller supplies a SID from a live native token/security descriptor/validated ACE.
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(denied());
    }
    // SAFETY: validated SID and initialized output; successful conversion allocates LocalAlloc text.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = decode_text(text);
    // SAFETY: successful conversion allocated text and decode does not retain it.
    unsafe {
        LocalFree(text.cast());
    }
    result
}

fn decode_text(text: *const u16) -> io::Result<String> {
    let mut count = 0;
    // SAFETY: SID conversion supplies terminated text; reject overlong values instead of truncating.
    unsafe {
        while count < SID_CHARS && *text.add(count) != 0 {
            count += 1;
        }
        if count == SID_CHARS {
            return Err(denied());
        }
        String::from_utf16(std::slice::from_raw_parts(text, count)).map_err(|_| denied())
    }
}

/// Verify by handle. Legacy ACLs may grant SYSTEM/Admin privileges; new resources grant only the user.
pub(super) fn verify(file: &File, strict: bool) -> io::Result<()> {
    let expected = current_user_sid()?;
    let mut owner = std::ptr::null_mut();
    let mut acl = std::ptr::null_mut();
    let mut descriptor = std::ptr::null_mut();
    // SAFETY: owned File and initialized outputs remain live; success returns one LocalAlloc descriptor.
    let error = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            std::ptr::null_mut(),
            &mut acl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if error != 0 {
        return Err(io::Error::from_raw_os_error(error as i32));
    }
    let held = Descriptor(descriptor);
    if descriptor.is_null() || sid_text(owner)? != expected || acl.is_null() {
        return Err(denied());
    }
    if strict {
        require_protected(&held)?;
    }
    verify_acl(acl, &expected, strict)
}

fn require_protected(descriptor: &Descriptor) -> io::Result<()> {
    let mut control = 0;
    let mut revision = 0;
    // SAFETY: descriptor guard owns valid security storage; scalar outputs are initialized.
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if control & SE_DACL_PROTECTED == 0 {
        return Err(denied());
    }
    Ok(())
}

fn verify_acl(acl: *const ACL, user: &str, strict: bool) -> io::Result<()> {
    let mut info = ACL_SIZE_INFORMATION::default();
    // SAFETY: GetSecurityInfo supplied this non-null ACL; output size matches the initialized structure.
    if unsafe {
        GetAclInformation(
            acl,
            (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
            AclSizeInformation,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if info.AceCount == 0 {
        return Err(denied());
    }
    for index in 0..info.AceCount {
        let mut ace = std::ptr::null_mut();
        // SAFETY: index is bounded by native ACL count; output points into the live descriptor.
        if unsafe { GetAce(acl, index, &mut ace) } == 0 {
            return Err(io::Error::last_os_error());
        }
        verify_ace(ace, user, strict)?;
    }
    Ok(())
}

fn verify_ace(ace: *const core::ffi::c_void, user: &str, strict: bool) -> io::Result<()> {
    if ace.is_null() {
        return Err(denied());
    }
    // SAFETY: GetAce returned an ACE from a live validated native ACL; header bounds subsequent fields.
    unsafe {
        let header = &*ace.cast::<ACE_HEADER>();
        if header.AceType != ALLOW_ACE
            || (header.AceSize as usize) < std::mem::size_of::<ACCESS_ALLOWED_ACE>()
        {
            return Err(denied());
        }
        let allowed = &*ace.cast::<ACCESS_ALLOWED_ACE>();
        let sid: PSID = std::ptr::addr_of!(allowed.SidStart).cast_mut().cast();
        // Bound SID sub-authorities inside this ACE before IsValidSid follows its variable-length tail.
        let offset = std::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart);
        let sid_bytes = header.AceSize as usize - offset;
        if sid_bytes < 8 || 8 + 4 * usize::from(*sid.cast::<u8>().add(1)) > sid_bytes {
            return Err(denied());
        }
        let text = sid_text(sid)?;
        // OS privilege principals cannot be excluded from administrative access; other users/groups are refused.
        if text != user && (strict || !matches!(text.as_str(), "S-1-5-18" | "S-1-5-32-544")) {
            return Err(denied());
        }
    }
    Ok(())
}
