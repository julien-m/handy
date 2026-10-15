//! Wire operation of `handy settings destination` (009, gate `destination-return` open).
use crate::cli::destination::DestinationAction;
use crate::settings_service::protocol::SettingsOperation;

/// Wire operation of a destination-domain action: no change option exists.
pub(super) fn destination_operation(action: &DestinationAction) -> SettingsOperation {
    let (name, arguments) = match action {
        DestinationAction::Pin { identifier } => ("pin", vec![identifier.clone()]),
        DestinationAction::Unpin => ("unpin", Vec::new()),
        DestinationAction::Status => ("status", Vec::new()),
    };
    SettingsOperation::Domain {
        domain: "destination".to_owned(),
        action: name.to_owned(),
        arguments,
        input: None,
        dry_run: false,
        expected_revision: None,
    }
}
