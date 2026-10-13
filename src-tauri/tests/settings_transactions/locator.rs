//! Native path-provider parity and portable marker reads never create or upgrade data.
use handy_app_lib::settings_service::locator::StoreLocator;
/// Parity with Tauri is not observed here: `StoreLocator::current` and Tauri's desktop path
/// resolver both call `dirs::data_dir()`, and Cargo.lock resolves a single `dirs 6.0.0` for both.
/// This test checks that the injected directory gets the identifier/store suffix and is not created.
#[test]
fn installed_resolution_appends_identifier_store_without_creating_it(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("handy");
    let installed = directory.path().join("installed");
    let path = StoreLocator::resolve(&executable, &installed)?;
    assert_eq!(
        path,
        directory
            .path()
            .canonicalize()?
            .join("installed/settings_store.json")
    );
    assert!(!installed.exists());
    let os = dirs::data_dir()
        .ok_or("OS data directory")?
        .join("com.pais.handy");
    let injected = StoreLocator::resolve(&executable, &os)?;
    assert_eq!(
        injected.file_name(),
        Some(std::ffi::OsStr::new("settings_store.json"))
    );
    assert!(injected.ends_with("com.pais.handy/settings_store.json"));
    Ok(())
}
#[test]
fn portable_magic_and_legacy_markers_are_read_without_mutation(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("handy");
    let marker = directory.path().join("portable");
    let data = directory.path().join("Data");
    let installed = directory.path().join("installed");
    std::fs::write(&marker, "Handy Portable Mode\n")?;
    let bytes = std::fs::read(&marker)?;
    assert_eq!(
        StoreLocator::resolve(&executable, &installed)?,
        directory
            .path()
            .canonicalize()?
            .join("Data/settings_store.json")
    );
    assert!(!data.exists());
    assert_eq!(std::fs::read(&marker)?, bytes);
    std::fs::write(&marker, [])?;
    assert!(
        StoreLocator::resolve(&executable, &installed)?.ends_with("installed/settings_store.json")
    );
    std::fs::create_dir(&data)?;
    assert!(StoreLocator::resolve(&executable, &installed)?.ends_with("Data/settings_store.json"));
    assert_eq!(std::fs::read(marker)?, b"");
    Ok(())
}
#[test]
fn missing_ancestor_is_canonicalized_and_parent_traversal_is_refused(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("handy");
    let root = directory.path().join("missing/child");
    assert_eq!(
        StoreLocator::resolve(&executable, &root)?,
        directory
            .path()
            .canonicalize()?
            .join("missing/child/settings_store.json")
    );
    assert!(!root.exists());
    assert!(StoreLocator::resolve(&executable, &directory.path().join("../foreign")).is_err());
    Ok(())
}
#[test]
fn unreadable_or_non_utf8_legacy_marker_beside_data_selects_portable_like_startup(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("handy");
    let marker = directory.path().join("portable");
    let installed = directory.path().join("installed");
    std::fs::write(&marker, [0xff, 0xfe, 0x00])?;
    assert!(
        StoreLocator::resolve(&executable, &installed)?.ends_with("installed/settings_store.json")
    );
    std::fs::create_dir(directory.path().join("Data"))?;
    assert!(StoreLocator::resolve(&executable, &installed)?.ends_with("Data/settings_store.json"));
    assert_eq!(std::fs::read(marker)?, [0xff, 0xfe, 0x00]);
    Ok(())
}
#[test]
#[cfg(unix)]
fn same_user_symlink_chain_resolves_hop_by_hop_to_its_final_directory(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let real = directory.path().join("real");
    std::fs::create_dir(&real)?;
    let middle = directory.path().join("middle");
    std::os::unix::fs::symlink(&real, &middle)?;
    let alias = directory.path().join("alias");
    std::os::unix::fs::symlink("middle", &alias)?;
    let executable = directory.path().join("handy");
    assert_eq!(
        StoreLocator::resolve(&executable, &alias.join("nested"))?,
        real.canonicalize()?.join("nested/settings_store.json")
    );
    Ok(())
}
#[test]
#[cfg(unix)]
fn symlink_loops_and_overlong_chains_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    use handy_app_lib::settings_service::errors::SettingsErrorCode;
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("handy");
    std::os::unix::fs::symlink("loop_b", directory.path().join("loop_a"))?;
    std::os::unix::fs::symlink("loop_a", directory.path().join("loop_b"))?;
    let looped = StoreLocator::resolve(&executable, &directory.path().join("loop_a/nested"))
        .err()
        .ok_or("symlink loop resolved")?;
    assert_eq!(looped.code(), SettingsErrorCode::Unavailable);
    std::fs::create_dir(directory.path().join("hop0"))?;
    for hop in 1..=10 {
        std::os::unix::fs::symlink(
            format!("hop{}", hop - 1),
            directory.path().join(format!("hop{hop}")),
        )?;
    }
    let overlong = StoreLocator::resolve(&executable, &directory.path().join("hop10/nested"))
        .err()
        .ok_or("overlong chain resolved")?;
    assert_eq!(overlong.code(), SettingsErrorCode::Unavailable);
    let short = StoreLocator::resolve(&executable, &directory.path().join("hop3/nested"))?;
    assert!(short.ends_with("hop0/nested/settings_store.json"));
    Ok(())
}
