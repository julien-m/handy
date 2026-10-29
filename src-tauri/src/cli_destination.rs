//! `handy settings destination` arguments (009, gate `destination-return` open only).
use clap::{Args, Subcommand};

/// `handy settings destination` (009): the session pin of the running Handy.
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "IDENTIFIER: bundle identifier (macOS), application user model ID or \
executable path (Windows) of a running application. The pin lasts until unpin or until Handy \
quits and changes the current dictation like the badge menu. Needs a running Handy (else \
unavailable, exit 5); unsupported on Linux (validation_failed, exit 2). Favorites (at most 10, \
newest nearest the badge): favorite-add needs a running Handy and a running application; \
favorite-remove works offline. Read them with handy settings get destination_favorites."
)]
pub struct DestinationArgs {
    #[command(subcommand)]
    pub action: DestinationAction,
}

/// Actions of the destination domain.
#[derive(Subcommand, Debug, Clone)]
pub enum DestinationAction {
    /// Pin the running application with this identifier
    Pin {
        /// Bundle identifier (macOS), AUMID or executable path (Windows)
        identifier: String,
    },
    /// Unpin: the next dictations go to the application they start in again
    Unpin,
    /// Print the pinned application, or null
    Status,
    /// Add the running application with this identifier to the favorites of the badge menu
    FavoriteAdd {
        /// Bundle identifier (macOS), AUMID or executable path (Windows)
        identifier: String,
    },
    /// Remove the application with this identifier from the favorites of the badge menu
    FavoriteRemove {
        /// Bundle identifier (macOS), AUMID or executable path (Windows)
        identifier: String,
    },
}
