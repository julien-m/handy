//! Compile-only proof imports exact production Windows modules and native standard traits.
#[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/windows_store.rs"]
mod windows_store;
#[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/instance_probe.rs"]
mod instance_probe;
fn main() {
    let path=std::path::Path::new(r"C:\Handy\Data");
    let _=windows_store::current_user_sid();
    let _=windows_store::ensure_private_parent(path);
    let _=windows_store::pin_parent(path).and_then(|parent| {parent.validate()?;let _=parent.path();Ok(())});
    let _=windows_store::open_checked(path,windows_store::ResourceKind::Directory,false);
    let _=windows_store::create_private_lock(&path.join("settings.lock")).and_then(|file| {fs2::FileExt::try_lock_exclusive(&file)?;let _=fs2::lock_contended_error().raw_os_error();Ok(())});
    let _=windows_store::open_private(&path.join("settings.lock"),true);
    if let (Ok(file),Ok(parent))=(windows_store::create_private(&path.join("settings.tmp")),windows_store::pin_parent(path)) {
        let _=windows_store::identity(&file);let result=windows_store::replace_owned(&file,&parent,std::ffi::OsStr::new("settings_store.json"));
        match result { Err(windows_store::ReplaceError::Before(error)|windows_store::ReplaceError::After(error)) => {let _=error.kind();}, Ok(())=>{} };
    }
    let _=instance_probe::inspect_legacy_windows();
    let _=instance_probe::classify_owner(Ok(false),instance_probe::ProbeScope::SameUser);
}
