//! Compile-only check of the production Windows field adapter of 011: the real
//! `destination/windows_field.rs`, `bounded_worker.rs`, `uia_mapping.rs` and `field.rs` are
//! compiled as a library for `x86_64-pc-windows-msvc` with the same `windows` crate version as
//! `src-tauri`. Only the crate paths they import are stubbed below, with the same signatures
//! (010's classifier types copied verbatim; 006's identity, dictation id and selection reduced
//! to the fields `field.rs` reads). It proves the adapter builds and passes clippy, not that it
//! works: the native Windows field return is pending release evidence.
#![deny(warnings)]

/// Stub of `crate::recovery::classifier` (types copied from `src-tauri`).
pub mod recovery {
    pub mod classifier {
        use std::time::Duration;
        /// Failure of one accessibility query (never carries a value).
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum AxQueryError {
            Timeout,
            NoValue,
            Unsupported,
            Failed,
        }
        /// Attributes whose settability shows an edit affordance.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum EditAttribute {
            Value,
            SelectedText,
            SelectedTextRange,
        }
        /// The destination's focused element, read with a timeout given per query.
        pub trait FocusedElement {
            fn pid(&mut self) -> Result<u32, AxQueryError>;
            fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError>;
            fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError>;
            fn is_settable(
                &mut self,
                attribute: EditAttribute,
                timeout: Duration,
            ) -> Result<bool, AxQueryError>;
        }
    }
}

/// The real 011 modules, with stubs of the 006 types `field.rs` imports.
pub mod destination;

/// Uses the adapter, so the check covers its public surface.
pub fn adapter() -> std::sync::Arc<dyn destination::field::FieldAccess> {
    std::sync::Arc::new(destination::windows_field::WindowsFieldAccess::new())
}
