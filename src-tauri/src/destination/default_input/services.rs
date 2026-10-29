//! Ports of feature 013 held by the destination manager: the remembered inputs (read from the
//! settings at each use), the accessibility tree, the command runner and the bundle of a running
//! instance. A manager built without them (every test, Windows, Linux) has no remembered input
//! and offers nothing, so every dictation behaves exactly as without this feature.
use super::model::{AddressingTool, DefaultInput};
use super::runner::{CliRunner, ProcessRunner};
use super::tree::{TreeAccess, UnsupportedTree};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Where a running instance's application bundle is.
pub trait BundleLocator: Send + Sync {
    /// The bundle directory of process `pid`, if it has one.
    fn bundle_path(&self, pid: u32) -> Option<PathBuf>;
}

/// No bundle anywhere (platforms without bundles, tests).
pub struct NoBundles;
impl BundleLocator for NoBundles {
    fn bundle_path(&self, _pid: u32) -> Option<PathBuf> {
        None
    }
}

/// Reads the remembered inputs (a settings snapshot).
pub type InputsSource = Box<dyn Fn() -> Vec<DefaultInput> + Send + Sync>;

/// The ports of feature 013.
pub struct DefaultInputServices {
    /// False where the feature is not offered (no menu entry, nothing applied).
    pub supported: bool,
    pub inputs: InputsSource,
    pub tree: Arc<dyn TreeAccess>,
    pub runner: Arc<dyn CliRunner>,
    pub bundles: Arc<dyn BundleLocator>,
}
impl DefaultInputServices {
    /// Services that offer nothing and remember nothing.
    pub fn none() -> Self {
        Self {
            supported: false,
            inputs: Box::new(Vec::new),
            tree: Arc::new(UnsupportedTree),
            runner: Arc::new(ProcessRunner),
            bundles: Arc::new(NoBundles),
        }
    }
    /// The remembered inputs now; none where the feature is not offered.
    pub fn remembered(&self) -> Vec<DefaultInput> {
        if !self.supported {
            return Vec::new();
        }
        (self.inputs)()
    }
    // @spec AC-001: Addressing tool found inside the bundle — .specs/features/013-default-input-per-app/spec.md#ac-001
    /// The addressing tool of process `pid` and its path: the first known tool present in the
    /// running instance's bundle (spec decision `capability_by_bundle`).
    pub fn tool_for(&self, pid: u32) -> Option<(AddressingTool, PathBuf)> {
        let bundle = self.bundles.bundle_path(pid)?;
        AddressingTool::ALL
            .into_iter()
            .find_map(|tool| tool_in_bundle(&bundle, tool).map(|path| (tool, path)))
    }
}

/// `tool`'s executable inside `bundle`: a regular executable file whose canonical path stays
/// inside the canonical bundle (a symbolic link leading out of the bundle is refused).
pub fn tool_in_bundle(bundle: &Path, tool: AddressingTool) -> Option<PathBuf> {
    let bundle = bundle.canonicalize().ok()?;
    let path = bundle
        .join(tool.bundle_relative_path())
        .canonicalize()
        .ok()?;
    let metadata = std::fs::metadata(&path).ok()?;
    (path.starts_with(&bundle) && metadata.is_file() && is_executable(&metadata)).then_some(path)
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}
#[cfg(not(unix))]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    // Windows has no execute bit; the feature is not offered there (spec decision `macos_only`).
    let _ = metadata;
    false
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::destination::default_input::fake_tree::{bundle_with_tool, OneBundle, TestResult};

    fn services(bundle: &Path) -> DefaultInputServices {
        DefaultInputServices {
            bundles: Arc::new(OneBundle(bundle.to_path_buf())),
            ..DefaultInputServices::none()
        }
    }

    // @spec AC-001: Addressing tool found inside the bundle — .specs/features/013-default-input-per-app/spec.md#ac-001
    #[test]
    fn an_executable_tool_inside_the_bundle_is_the_capability() -> TestResult {
        let bundle = bundle_with_tool(0o755)?;
        let found = services(bundle.path()).tool_for(7);
        assert_eq!(found.map(|(tool, _)| tool), Some(AddressingTool::Cmux));
        Ok(())
    }

    #[test]
    fn a_missing_or_non_executable_tool_is_no_capability() -> TestResult {
        let plain = tempfile::tempdir().map_err(|error| format!("tempdir: {error:?}"))?;
        assert!(services(plain.path()).tool_for(7).is_none());
        let bundle = bundle_with_tool(0o644)?;
        assert!(services(bundle.path()).tool_for(7).is_none());
        Ok(())
    }

    #[test]
    fn a_link_leading_out_of_the_bundle_is_refused() -> TestResult {
        let outside = bundle_with_tool(0o755)?;
        let bundle = tempfile::tempdir().map_err(|error| format!("tempdir: {error:?}"))?;
        let link = bundle
            .path()
            .join(AddressingTool::Cmux.bundle_relative_path());
        std::fs::create_dir_all(link.parent().ok_or("parent")?)
            .map_err(|error| format!("dirs: {error:?}"))?;
        let target = outside
            .path()
            .join(AddressingTool::Cmux.bundle_relative_path());
        std::os::unix::fs::symlink(target, &link).map_err(|error| format!("link: {error:?}"))?;
        assert!(services(bundle.path()).tool_for(7).is_none());
        Ok(())
    }

    #[test]
    fn unsupported_services_remember_nothing() -> TestResult {
        let mut services = DefaultInputServices::none();
        services.inputs = Box::new(|| {
            vec![DefaultInput {
                identity: crate::destination::context::ApplicationIdentity::BundleId(
                    "x".to_owned(),
                ),
                display_name: "X".to_owned(),
                target: super::super::model::DefaultTarget::Terminal(
                    super::super::model::TerminalTarget {
                        tool: AddressingTool::Cmux,
                        workspace_id: "AB".to_owned(),
                        workspace_name: "w".to_owned(),
                    },
                ),
            }]
        });
        assert!(services.remembered().is_empty());
        services.supported = true;
        assert_eq!(services.remembered().len(), 1);
        Ok(())
    }
}
