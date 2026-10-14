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
    /// Shortcuts of each action: list them; add, remove or reset them in the running Handy
    Shortcuts(ShortcutsArgs),
    /// AI post-processing prompts: list them with their readiness (running Handy); create,
    /// update, delete, choose the default and the provider override
    Prompts(PromptsArgs),
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

/// Change options shared by the shortcut actions.
#[derive(Args, Debug, Clone, Default)]
pub struct ChangeOptions {
    /// Refuse the change unless the settings are still at revision N
    #[arg(long, value_name = "N")]
    pub expected_revision: Option<u64>,
    /// Validate and show the change without writing or registering it
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

impl CliArgs {
    /// Return whether the GUI startup path may detach its inherited Windows console.
    // @spec FR-001: Preserve headless console output — .specs/features/008-settings-cli/spec.md#fr-001
    pub fn should_detach_console(&self) -> bool {
        self.transcribe_file.is_none() && !self.list_devices && !self.list_models
    }
}
