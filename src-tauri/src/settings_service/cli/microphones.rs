//! Wire operation of `handy settings microphones` (feature 001).
use crate::cli::{ChangeOptions, MicrophonesAction};
use crate::settings_service::protocol::{Revision, SettingsOperation};

/// Wire operation of a microphones-domain action.
pub(super) fn microphones_operation(action: &MicrophonesAction) -> SettingsOperation {
    let none = ChangeOptions::default();
    let (name, arguments, options) = match action {
        MicrophonesAction::List => ("list", Vec::new(), &none),
        MicrophonesAction::SetPriority { names, options } => {
            ("set-priority", names.clone(), options)
        }
    };
    SettingsOperation::Domain {
        domain: "microphones".to_owned(),
        action: name.to_owned(),
        arguments,
        input: None,
        dry_run: options.dry_run,
        expected_revision: options.expected_revision.map(Revision),
    }
}
