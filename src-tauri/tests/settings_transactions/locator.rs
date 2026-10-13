//! Native path-provider parity and portable marker reads never create or upgrade data.
use handy_app_lib::settings_service::locator::StoreLocator;
#[test]
fn installed_resolution_uses_the_same_directory_provider_as_tauri(
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
