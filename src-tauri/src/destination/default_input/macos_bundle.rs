//! macOS bundle of a running application (feature 013): `NSRunningApplication.bundleURL` of a
//! process, where the addressing tool of its type is looked for. A local read, no messaging.
use super::services::BundleLocator;
use objc2::rc::autoreleasepool;
use objc2_app_kit::NSRunningApplication;
use std::path::PathBuf;

/// Production [`BundleLocator`] of macOS.
pub struct MacBundles;

impl BundleLocator for MacBundles {
    fn bundle_path(&self, pid: u32) -> Option<PathBuf> {
        let pid = i32::try_from(pid).ok()?;
        autoreleasepool(|_| {
            let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
            let path = app.bundleURL()?.path()?;
            Some(PathBuf::from(path.to_string()))
        })
    }
}
