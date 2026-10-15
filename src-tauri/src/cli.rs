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
    /// Output settings: paste method, clipboard handling, automatic submit, delays, typing tool,
    /// return to the dictation's application
    Output(OutputArgs),
    /// Word replacement rules: list, add, update, remove, import and export
    Replacements(ReplacementsArgs),
    /// Shortcuts of each action: list them; add, remove or reset them in the running Handy
    Shortcuts(ShortcutsArgs),
    /// Recognition vocabulary: list, add, remove, import or export terms; status for the
    /// selected model
    Vocabulary(VocabularyArgs),
    /// AI post-processing prompts: list them with their readiness (running Handy); create,
    /// update, delete, choose the default and the provider override
    Prompts(PromptsArgs),
    /// Microphone choices: list the saved order, availability and microphone in use; replace it
    Microphones(MicrophonesArgs),
    /// Pinned destination application (running Handy): pin one by identifier, unpin, status
    #[cfg(feature = "destination-return")]
    Destination(DestinationArgs),
}

/// `handy settings destination` (009): the session pin of the running Handy.
#[cfg(feature = "destination-return")]
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "IDENTIFIER: bundle identifier (macOS), application user model ID or \
executable path (Windows) of a running application. The pin lasts until unpin or until Handy \
quits and changes the current dictation like the badge menu. Needs a running Handy (else \
unavailable, exit 5); unsupported on Linux (validation_failed, exit 2)."
)]
pub struct DestinationArgs {
    #[command(subcommand)]
    pub action: DestinationAction,
}

/// Actions of the destination domain.
#[cfg(feature = "destination-return")]
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
}

/// `handy settings vocabulary`: the recognition vocabulary, with the validation of the settings
/// window.
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "TERM has 1 to 100 characters, no line break and none of < > \". Terms are \
unique ignoring case and spacing; a list holds at most 500 terms. export prints one term per line \
on stdout: redirect it to save a file. status needs a running Handy, which reports the model it \
has loaded; without one it fails with unavailable (exit 5)."
)]
pub struct VocabularyArgs {
    #[command(subcommand)]
    pub action: VocabularyAction,
}

/// How an import combines with the saved terms.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocabularyImportMode {
    /// Keep the saved terms and append the new ones in input order
    Merge,
    /// Make the input the whole list, in input order
    Replace,
}

/// Actions of the vocabulary domain.
#[derive(Subcommand, Debug, Clone)]
pub enum VocabularyAction {
    /// Print the saved terms in order, with the reason a term is not sent to recognition
    List,
    /// Append one term
    Add {
        /// Term to add
        term: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Remove the first saved term equal to TERM, ignoring case and spacing
    Remove {
        /// Term to remove
        term: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Import terms, one per line, as one change (duplicates skipped, any invalid line refuses
    /// the whole input)
    Import {
        /// File path, or `-` for stdin
        source: String,
        /// merge appends the new terms; replace makes the input the whole list
        #[arg(long, value_enum, default_value_t = VocabularyImportMode::Merge)]
        mode: VocabularyImportMode,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Print one saved term per line on stdout
    Export {
        /// `-` for stdout (the only target)
        #[arg(default_value = "-")]
        target: String,
    },
    /// Whether the selected model uses the vocabulary during recognition, with the terms sent
    /// and not sent (needs a running Handy)
    Status,
}

/// `handy settings prompts`: the prompt library, with the validation of the settings window.
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "A name has 1 to 60 characters and is unique ignoring case; a text has 1 to 8000 \
characters and is not only spaces; a library holds at most 50 prompts. TEXT - reads the text \
from stdin. list needs a running Handy (it checks provider readiness there); without one it fails \
with unavailable, and the library stays readable with handy settings get post_process_prompts. \
Deleting a prompt also deletes its shortcuts (manage them with handy settings shortcuts and the \
action prompt:ID); during a dictation that deletion fails with unavailable (reason \
shortcut_session_busy). API keys are never printed; configure them in the settings window."
)]
pub struct PromptsArgs {
    #[command(subcommand)]
    pub action: PromptsAction,
}
/// Actions of the prompts domain.
#[derive(Subcommand, Debug, Clone)]
pub enum PromptsAction {
    /// Print every prompt with its shortcuts, provider readiness and the default prompt
    List,
    /// Create a prompt
    Create {
        /// Name, unique ignoring case
        #[arg(long)]
        name: String,
        /// Text; "${output}" stands for the transcript; - reads it from stdin
        #[arg(long)]
        text: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Change the name and/or the text of prompt ID
    #[command(visible_alias = "edit")]
    Update {
        /// Prompt id, as listed
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New text; - reads it from stdin
        #[arg(long)]
        text: Option<String>,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Delete prompt ID and its shortcuts; a deleted default becomes none
    #[command(visible_alias = "remove")]
    Delete {
        /// Prompt id, as listed
        id: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Choose the default prompt of the post-processing shortcut (a prompt id, or none)
    SetDefault {
        /// Prompt id, or none
        id: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Set the provider and model of prompt ID, or --clear to use the default provider again
    SetProviderOverride {
        /// Prompt id, as listed
        id: String,
        /// Provider id (openai, custom, apple_intelligence, ...)
        #[arg(long, requires = "model", required_unless_present = "clear")]
        provider: Option<String>,
        /// Model of that provider
        #[arg(long, requires = "provider")]
        model: Option<String>,
        /// Use the default provider and model again
        #[arg(long, conflicts_with_all = ["provider", "model"])]
        clear: bool,
        #[command(flatten)]
        options: ChangeOptions,
    },
}

/// `handy settings microphones`: the ordered microphone choices (feature 001).
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Recording tries the choices in order and uses the first one present, then the \
system default; unavailable choices stay saved. list needs a running Handy (it reads the devices \
and the microphone in use); without one it fails with unavailable and `handy settings get \
microphone_priority` still prints the saved order. set-priority replaces the whole order, with \
or without a running Handy; no NAME sets the empty order (system default). Quote names with \
spaces; put -- before a name starting with a hyphen."
)]
pub struct MicrophonesArgs {
    #[command(subcommand)]
    pub action: MicrophonesAction,
}

/// Actions of the microphones domain.
#[derive(Subcommand, Debug, Clone)]
pub enum MicrophonesAction {
    /// Print the saved order, each choice's availability and the microphone in use
    List,
    /// Replace the whole order of microphone choices, first choice first
    SetPriority {
        /// Exact device names, as listed by the settings window or the system
        names: Vec<String>,
        #[command(flatten)]
        options: ChangeOptions,
    },
}

/// `handy settings shortcuts`: the shortcut lists, with the validation of the settings window.
#[derive(Args, Debug, Clone)]
#[command(
    after_help = "SHORTCUT uses the syntax of the active keyboard implementation, as listed (for \
example ctrl+option+d). add, remove and reset need a running Handy, which registers the change \
at once; without one they fail with unavailable (reason instance_required). They also fail with \
unavailable (reason shortcut_session_busy) while a dictation is in progress."
)]
pub struct ShortcutsArgs {
    #[command(subcommand)]
    pub action: ShortcutsAction,
}

/// Change options shared by the shortcut and replacement actions that write.
#[derive(Args, Debug, Clone, Copy, Default)]
pub struct ChangeOptions {
    /// Refuse the change unless the settings are still at revision N
    #[arg(long, value_name = "N")]
    pub expected_revision: Option<u64>,
    /// Validate and show the change without writing it (or registering a shortcut)
    #[arg(long)]
    pub dry_run: bool,
}

/// Actions of the shortcuts domain.
#[derive(Subcommand, Debug, Clone)]
pub enum ShortcutsAction {
    /// Print the shortcuts of every action, or only of ACTION
    List {
        /// Action id (transcribe, transcribe_with_post_process, cancel, ...)
        action: Option<String>,
    },
    /// Add an alternative shortcut to ACTION (dictation actions, at most 4 shortcuts)
    Add {
        /// Action id
        action: String,
        /// Shortcut to add, in the active implementation's syntax
        shortcut: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Remove one shortcut of ACTION (the last one cannot be removed)
    Remove {
        /// Action id
        action: String,
        /// Stored shortcut to remove (same combination in any modifier order)
        shortcut: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
    /// Restore the single default shortcut of ACTION
    Reset {
        /// Action id
        action: String,
        #[command(flatten)]
        options: ChangeOptions,
    },
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
