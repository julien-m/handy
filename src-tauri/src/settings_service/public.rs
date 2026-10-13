//! Explicit field/nested DTO projection; AppSettings/SecretMap are never serialized here.

use crate::settings::*;
use serde::Serialize;
use std::collections::BTreeMap;

/// Complete non-secret field allowlist, flattened for the settings get contract.
// @spec FR-002: Redacted public settings projection — .specs/features/008-settings-cli/spec.md#fr-002
#[derive(Serialize)]
pub struct PublicSettings {
    #[serde(flatten)]
    general: PublicGeneral,
    #[serde(flatten)]
    advanced: PublicAdvanced,
    #[serde(flatten)]
    readonly: PublicReadonly,
    #[serde(flatten)]
    collections: PublicCollections,
}
impl PublicSettings {
    /// Transform a typed internal snapshot through explicit field-level projections.
    pub(crate) fn from_settings(settings: &AppSettings) -> Self {
        Self {
            general: PublicGeneral::from_settings(settings),
            advanced: PublicAdvanced::from_settings(settings),
            readonly: PublicReadonly::from_settings(settings),
            collections: PublicCollections::from_settings(settings),
        }
    }
}

#[derive(Serialize)]
struct PublicGeneral {
    shortcut_activation: ShortcutActivation,
    hold_threshold_ms: u64,
    audio_feedback: bool,
    audio_feedback_volume: f32,
    sound_theme: SoundTheme,
    start_hidden: bool,
    autostart_enabled: bool,
    update_checks_enabled: bool,
    show_whats_new_on_update: bool,
    always_on_microphone: bool,
    selected_microphone: Option<String>,
    selected_channel: Option<u16>,
    clamshell_microphone: Option<String>,
    selected_output_device: Option<String>,
    translate_to_english: bool,
    selected_language: String,
    overlay_position: OverlayPosition,
    debug_mode: bool,
    log_level: LogLevel,
    model_unload_timeout: ModelUnloadTimeout,
    word_correction_threshold: f64,
    history_limit: usize,
    recording_retention_period: RecordingRetentionPeriod,
    paste_method: PasteMethod,
    clipboard_handling: ClipboardHandling,
    auto_submit: bool,
}
impl PublicGeneral {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            shortcut_activation: settings.shortcut_activation,
            hold_threshold_ms: settings.hold_threshold_ms,
            audio_feedback: settings.audio_feedback,
            audio_feedback_volume: settings.audio_feedback_volume,
            sound_theme: settings.sound_theme,
            start_hidden: settings.start_hidden,
            autostart_enabled: settings.autostart_enabled,
            update_checks_enabled: settings.update_checks_enabled,
            show_whats_new_on_update: settings.show_whats_new_on_update,
            always_on_microphone: settings.always_on_microphone,
            selected_microphone: settings.selected_microphone.clone(),
            selected_channel: settings.selected_channel,
            clamshell_microphone: settings.clamshell_microphone.clone(),
            selected_output_device: settings.selected_output_device.clone(),
            translate_to_english: settings.translate_to_english,
            selected_language: settings.selected_language.clone(),
            overlay_position: settings.overlay_position,
            debug_mode: settings.debug_mode,
            log_level: settings.log_level,
            model_unload_timeout: settings.model_unload_timeout,
            word_correction_threshold: settings.word_correction_threshold,
            history_limit: settings.history_limit,
            recording_retention_period: settings.recording_retention_period,
            paste_method: settings.paste_method,
            clipboard_handling: settings.clipboard_handling,
            auto_submit: settings.auto_submit,
        }
    }
}

#[derive(Serialize)]
struct PublicAdvanced {
    auto_submit_key: AutoSubmitKey,
    post_process_enabled: bool,
    post_process_selected_prompt_id: Option<String>,
    mute_while_recording: bool,
    append_trailing_space: bool,
    app_language: String,
    theme: Theme,
    experimental_enabled: bool,
    lazy_stream_close: bool,
    keyboard_implementation: KeyboardImplementation,
    show_tray_icon: bool,
    paste_delay_ms: u64,
    paste_delay_after_ms: u64,
    reliable_paste: bool,
    typing_tool: TypingTool,
    external_script_path: Option<String>,
    filler_word_removal_enabled: bool,
    chinese_script: ChineseScript,
    transcribe_accelerator: TranscribeAcceleratorSetting,
    ort_accelerator: OrtAcceleratorSetting,
    transcribe_gpu_device: Option<String>,
    extra_recording_buffer_ms: u64,
    vad_enabled: bool,
    vad_backend: VadBackend,
    overlay_style: OverlayStyle,
}
impl PublicAdvanced {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            auto_submit_key: settings.auto_submit_key,
            post_process_enabled: settings.post_process_enabled,
            post_process_selected_prompt_id: settings.post_process_selected_prompt_id.clone(),
            mute_while_recording: settings.mute_while_recording,
            append_trailing_space: settings.append_trailing_space,
            app_language: settings.app_language.clone(),
            theme: settings.theme,
            experimental_enabled: settings.experimental_enabled,
            lazy_stream_close: settings.lazy_stream_close,
            keyboard_implementation: settings.keyboard_implementation,
            show_tray_icon: settings.show_tray_icon,
            paste_delay_ms: settings.paste_delay_ms,
            paste_delay_after_ms: settings.paste_delay_after_ms,
            reliable_paste: settings.reliable_paste,
            typing_tool: settings.typing_tool,
            external_script_path: settings.external_script_path.clone(),
            filler_word_removal_enabled: settings.filler_word_removal_enabled,
            chinese_script: settings.chinese_script,
            transcribe_accelerator: settings.transcribe_accelerator,
            ort_accelerator: settings.ort_accelerator,
            transcribe_gpu_device: settings.transcribe_gpu_device.clone(),
            extra_recording_buffer_ms: settings.extra_recording_buffer_ms,
            vad_enabled: settings.vad_enabled,
            vad_backend: settings.vad_backend,
            overlay_style: settings.overlay_style,
        }
    }
}

#[derive(Serialize)]
struct PublicReadonly {
    settings_schema_version: u32,
    whats_new_last_seen_version: String,
    selected_model: String,
    onboarding_completed: bool,
    post_process_provider_id: String,
    post_process_providers: Vec<PublicProvider>,
    post_process_api_keys: BTreeMap<String, SecretPresence>,
    post_process_models: BTreeMap<String, String>,
}
impl PublicReadonly {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            settings_schema_version: settings.settings_schema_version,
            whats_new_last_seen_version: settings.whats_new_last_seen_version.clone(),
            selected_model: settings.selected_model.clone(),
            onboarding_completed: settings.onboarding_completed,
            post_process_provider_id: settings.post_process_provider_id.clone(),
            post_process_providers: settings
                .post_process_providers
                .iter()
                .map(PublicProvider::from)
                .collect(),
            // Intentionally discard each raw key; only its non-empty presence crosses this boundary.
            post_process_api_keys: settings
                .post_process_api_keys
                .iter()
                .map(|(id, value)| {
                    (
                        id.clone(),
                        SecretPresence {
                            present: !value.is_empty(),
                        },
                    )
                })
                .collect(),
            post_process_models: settings
                .post_process_models
                .iter()
                .map(|(id, value)| (id.clone(), value.clone()))
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct PublicCollections {
    bindings: BTreeMap<String, PublicBinding>,
    custom_words: Vec<String>,
    post_process_prompts: Vec<PublicPrompt>,
    custom_filler_words: Option<Vec<String>>,
}
impl PublicCollections {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            bindings: settings
                .bindings
                .iter()
                .map(|(id, value)| (id.clone(), PublicBinding::from(value)))
                .collect(),
            custom_words: settings.custom_words.clone(),
            post_process_prompts: settings
                .post_process_prompts
                .iter()
                .map(PublicPrompt::from)
                .collect(),
            custom_filler_words: settings.custom_filler_words.clone(),
        }
    }
}

#[derive(Serialize)]
struct SecretPresence {
    present: bool,
}
#[derive(Serialize)]
struct PublicProvider {
    id: String,
    label: String,
    base_url: String,
    allow_base_url_edit: bool,
    models_endpoint: Option<String>,
    supports_structured_output: bool,
}
impl From<&PostProcessProvider> for PublicProvider {
    fn from(value: &PostProcessProvider) -> Self {
        Self {
            id: value.id.clone(),
            label: value.label.clone(),
            base_url: value.base_url.clone(),
            allow_base_url_edit: value.allow_base_url_edit,
            models_endpoint: value.models_endpoint.clone(),
            supports_structured_output: value.supports_structured_output,
        }
    }
}
#[derive(Serialize)]
struct PublicPrompt {
    id: String,
    name: String,
    prompt: String,
}
impl From<&LLMPrompt> for PublicPrompt {
    fn from(value: &LLMPrompt) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            prompt: value.prompt.clone(),
        }
    }
}
#[derive(Serialize)]
struct PublicBinding {
    id: String,
    name: String,
    description: String,
    default_binding: String,
    current_binding: String,
}
impl From<&ShortcutBinding> for PublicBinding {
    fn from(value: &ShortcutBinding) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            default_binding: value.default_binding.clone(),
            current_binding: value.current_binding.clone(),
        }
    }
}
