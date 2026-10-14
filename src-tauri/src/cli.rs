//! Parse Handy startup, recording-control, headless transcription and settings arguments.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "handy", about = "Handy - Speech to Text")]
pub struct CliArgs {
    /// Start with the main window hidden
    #[arg(long)]
    pub start_hidden: bool,

    /// Disable the system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Toggle transcription on/off (sent to running instance)
    #[arg(long)]
    pub toggle_transcription: bool,

    /// Toggle transcription with post-processing on/off (sent to running instance)
    #[arg(long)]
    pub toggle_post_process: bool,

    /// Cancel the current operation (sent to running instance)
    #[arg(long)]
    pub cancel: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    pub debug: bool,

    /// Transcribe this WAV (16 kHz mono) headlessly and exit. Runs the same
    /// batch transcription path as the app — no mic, no VAD, no download
    /// (the model must already be installed).
    #[arg(short = 'f', long, value_name = "WAV")]
    pub transcribe_file: Option<PathBuf>,

    /// Model id to load for --transcribe-file (default: the selected model).
    #[arg(long)]
    pub model: Option<String>,

    /// Hard-select the compute device for --transcribe-file by its registry
    /// index (see --list-devices). Omit to use the persisted accelerator
    /// setting. transcribe-cpp (whisper-family) models only.
    #[arg(long, value_name = "N")]
    pub device_index: Option<usize>,

    /// List the transcribe-cpp compute devices (with indices) and exit.
    #[arg(long)]
    pub list_devices: bool,

    /// List the available models (with ids) and exit. Pass an id to --model.
    /// Honors --json for machine-readable output.
    #[arg(long)]
    pub list_models: bool,

    /// Repeat the transcription N times (best_ms reports the fastest run).
    #[arg(long, value_name = "N")]
    pub repeat: Option<usize>,

    /// Emit --transcribe-file results as JSON.
    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

/// Subcommands; their absence keeps every historical invocation unchanged.
#[derive(Subcommand, Debug, Clone)]
pub enum CliCommand {
    /// Read and change Handy's persistent settings without starting the app
    Settings(SettingsArgs),
}

/// `handy settings`: one operation, routed to a running Handy or applied to the store.
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Exit codes: 0 success; 1 persistence_failed, internal_error; 2 usage_error, \
validation_failed, shortcut_conflict, limit_exceeded; 3 revision_conflict; 4 not_found; \
5 unavailable.\n\
After an unavailable error with outcome \"unknown\", read the settings again before retrying."
)]
pub struct SettingsArgs {
    /// Print exactly one JSON envelope on stdout (also accepted before `settings`)
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub operation: SettingsCommand,
}

/// Generic settings operations of this build.
#[derive(Subcommand, Debug, Clone)]
pub enum SettingsCommand {
    /// Print all non-secret settings, or only KEY
    Get {
        /// Setting key, as listed by `handy settings keys`
        key: Option<String>,
    },
    /// List settings keys with type, allowed values or range, and read-only flag
    Keys,
    /// Change one scalar setting (true/false, integer, number, allowed value, text or null)
    #[command(allow_negative_numbers = true)]
    Set {
        /// Setting key, as listed by `handy settings keys`
        key: String,
        /// New value, parsed with the key's type
        value: String,
        /// Refuse the change unless the settings are still at revision N
        #[arg(long, value_name = "N")]
        expected_revision: Option<u64>,
    },
    /// Output settings: paste method, clipboard handling, automatic submit, delays, typing tool
    Output(OutputArgs),
    /// Word replacement rules: list, add, update, remove, import and export
    Replacements(ReplacementsArgs),
}

/// `handy settings output`: the existing output settings, with the validation of `set`.
#[derive(Args, Debug, Clone)]
pub struct OutputArgs {
    #[command(subcommand)]
    pub action: OutputAction,
}

/// Actions of the output domain in this build.
#[derive(Subcommand, Debug, Clone)]
pub enum OutputAction {
    /// Print the output settings, or only KEY
    Get {
        /// Output setting key (paste_method, clipboard_handling, auto_submit, ...)
        key: Option<String>,
    },
    /// Change one output setting; with --dry-run, validate and preview without writing
    #[command(allow_negative_numbers = true)]
    Set {
        /// Output setting key
        key: String,
        /// New value, parsed with the key's type
        value: String,
        /// Refuse the change unless the settings are still at revision N
        #[arg(long, value_name = "N")]
        expected_revision: Option<u64>,
        /// Validate and show the change without writing it
        #[arg(long)]
        dry_run: bool,
    },
}

/// `handy settings replacements`: ordered rules applied to every dictation (exchange format:
/// `recognized;replacement` or `recognized<TAB>replacement`, one rule per line).
#[derive(Args, Debug, Clone)]
pub struct ReplacementsArgs {
    #[command(subcommand)]
    pub action: ReplacementsAction,
}

/// Change options shared by the replacement actions that write.
#[derive(Args, Debug, Clone, Copy, Default)]
pub struct ChangeOptions {
    /// Refuse the change unless the settings are still at revision N
    #[arg(long, value_name = "N")]
    pub expected_revision: Option<u64>,
    /// Validate and show the change without writing it
    #[arg(long)]
    pub dry_run: bool,
}

/// How an import combines with the saved rules.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportModeArg {
    /// Update the replacement of rules with the same recognized text, append the new ones
    Merge,
    /// Make the input the whole rule set, in input order
    Replace,
}

/// Actions of the replacements domain.
#[derive(Subcommand, Debug, Clone)]
pub enum ReplacementsAction {
    /// Print the rules in their saved order (id, recognized text, replacement)
    List,
    /// Append a rule
    Add {
        /// Text as recognized by the speech model (1-100 characters, no `;`, tab or line break)
        recognized: String,
        /// Text inserted instead, exactly as written (1-500 characters, no tab or line break)
        replacement: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Change both texts of a rule, keeping its id and position
    #[command(alias = "edit")]
    Update {
        /// Rule id, as printed by `list`
        id: String,
        /// New recognized text
        recognized: String,
        /// New replacement
        replacement: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Remove a rule
    #[command(alias = "delete")]
    Remove {
        /// Rule id, as printed by `list`
        id: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Import rules from a file, or `-` for stdin (UTF-8, at most 1,000,000 bytes), atomically
    Import {
        /// File path, or `-` to read stdin
        source: String,
        /// merge: update matching rules and append new ones; replace: the input becomes the set
        #[arg(long, value_enum)]
        mode: ImportModeArg,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Export the rules to a file, or `-` (default) for stdout
    Export {
        /// File path, or `-` for stdout
        #[arg(default_value = "-")]
        destination: String,
    },
}

impl CliArgs {
    /// Return whether the GUI startup path may detach its inherited Windows console.
    // @spec FR-001: Preserve headless console output — .specs/features/008-settings-cli/spec.md#fr-001
    pub fn should_detach_console(&self) -> bool {
        self.transcribe_file.is_none() && !self.list_devices && !self.list_models
    }
}
