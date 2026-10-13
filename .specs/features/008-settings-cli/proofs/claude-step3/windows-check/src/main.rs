//! Compile-only check of the exact production atomic/locator/store Windows branches with stubbed neighbours.
#![allow(dead_code)]
mod settings {
    #[derive(Default, serde::Serialize, serde::Deserialize)]
    pub struct Prompt { pub id: String }
    #[derive(Default, serde::Serialize, serde::Deserialize)]
    pub struct AppSettings { pub post_process_prompts: Vec<Prompt> }
    pub fn get_default_settings() -> AppSettings { AppSettings::default() }
    pub fn update_checks_forced_disabled() -> bool { false }
}
mod portable {
    pub(crate) fn resolve_data_dir(_: &std::path::Path, _: &std::path::Path) -> Result<Option<std::path::PathBuf>, crate::settings_service::errors::SettingsError> { Ok(None) }
}
mod settings_service {
    pub mod errors {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SettingsErrorCode { Unavailable, PersistenceFailed }
        #[derive(Debug)]
        pub struct SettingsError(pub SettingsErrorCode);
        impl SettingsError {
            pub fn new(code: SettingsErrorCode) -> Self { Self(code) }
            pub fn unknown_outcome() -> Self { Self(SettingsErrorCode::Unavailable) }
            pub fn code(&self) -> SettingsErrorCode { self.0 }
            pub fn revision_conflict(_: super::protocol::Revision) -> Self { Self(SettingsErrorCode::Unavailable) }
        }
    }
    pub mod protocol {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Revision(pub u64);
        pub struct SettingsSnapshot { pub settings: crate::settings::AppSettings, pub revision: Revision }
    }
    pub mod normalization {
        pub(crate) fn normalize(_: &serde_json::Value) -> Result<(crate::settings::AppSettings, bool), super::errors::SettingsError> { Ok((Default::default(), false)) }
    }
    pub mod ownership {
        use super::errors::SettingsError;
        pub(crate) struct FileLock;
        impl FileLock {
            pub fn acquire(_: std::path::PathBuf, _: bool) -> Result<Self, SettingsError> { Ok(Self) }
            pub fn validate(&self) -> Result<(), SettingsError> { Ok(()) }
        }
        pub trait OwnerProbe { fn inspect(&self) -> bool; }
        pub struct NativeOwnerProbe;
        impl OwnerProbe for NativeOwnerProbe { fn inspect(&self) -> bool { false } }
        pub struct StartupLease { pub(crate) lock: FileLock }
        pub(crate) fn require_absent(_: &impl OwnerProbe) -> Result<(), SettingsError> { Ok(()) }
    }
    pub mod descriptor {
        pub struct SettingDescriptor { pub key: &'static str, pub secret: bool }
        static ALL: [SettingDescriptor; 1] = [SettingDescriptor { key: "post_process_prompts", secret: false }];
        pub fn descriptors() -> &'static [SettingDescriptor] { &ALL }
        pub fn descriptor(key: &str) -> Option<&'static SettingDescriptor> { ALL.iter().find(|entry| entry.key == key) }
    }
    pub mod validation {
        use super::errors::SettingsError;
        pub struct SettingsMutation;
        impl SettingsMutation {
            pub fn key(&self) -> &'static str { "post_process_prompts" }
            pub fn value(&self) -> serde_json::Value { serde_json::json!([]) }
        }
        pub struct ValidationContext<'a> { pub prompt_ids: &'a [&'a str], pub updates_allowed: bool }
        pub(crate) fn validate_value(_: &'static super::descriptor::SettingDescriptor, _: serde_json::Value, _: &ValidationContext<'_>) -> Result<SettingsMutation, SettingsError> { Ok(SettingsMutation) }
    }
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/transaction.rs"]
    pub mod transaction;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/raw_merge.rs"]
    pub mod raw_merge;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/atomic.rs"]
    pub mod atomic;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/locator.rs"]
    pub mod locator;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/store.rs"]
    pub mod store;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/windows_store.rs"]
    pub mod windows_store;
    pub fn exercise(path: &std::path::Path) {
        let _ = atomic::persist(path, &serde_json::json!({}), &(), || Ok(()));
        let _ = locator::StoreLocator::current();
        let _ = locator::canonical_path(path);
        let repository = store::SettingsRepository::new(path.to_path_buf());
        let _ = repository.commit_mutation(validation::SettingsMutation, None);
        let _ = repository.initialize(&ownership::NativeOwnerProbe);
        let _ = repository.read_snapshot();
        let _ = repository.load_at(path).map(|document| (document.raw, document.writable, document.missing, document.read.warning));
    }
}
fn main() {
    let path = std::path::Path::new(r"C:\Handy\Data\settings_store.json");
    settings_service::exercise(path);
}
