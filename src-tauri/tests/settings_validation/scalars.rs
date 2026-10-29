//! Independent malformed-value and boundary assertions; no actual writer parity claim.

use handy_app_lib::settings_service::platform_capability::{
    PlatformCapabilities, PlatformCapability,
};
use handy_app_lib::settings_service::{
    errors::{SettingsError, SettingsErrorCode},
    validation::{validate_cli, validate_ui, SettingsMutation, ValidationContext},
};
use serde_json::json;

#[test]
fn cli_rejects_noncanonical_boolean_and_out_of_range_numbers(
) -> Result<(), Box<dyn std::error::Error>> {
    let context = ValidationContext {
        prompt_ids: &["known"],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    for (key, value) in [
        ("audio_feedback", "TRUE"),
        ("hold_threshold_ms", "99"),
        ("paste_delay_ms", "5001"),
        ("audio_feedback_volume", "NaN"),
        ("log_level", "3"),
        ("overlay_position", "none"),
    ] {
        let error = validate_cli(key, value, &context)
            .err()
            .ok_or_else(|| "invalid input accepted".to_owned())?;
        assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
    }
    Ok(())
}
#[test]
fn unknown_and_protected_keys_are_refused_without_echoing_input(
) -> Result<(), Box<dyn std::error::Error>> {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    let secret = "CANARY_NOT_A_SETTING_SECRET";
    let error = validate_cli(secret, secret, &context)
        .err()
        .ok_or_else(|| "unknown key accepted".to_owned())?;
    assert_eq!(error.code(), SettingsErrorCode::NotFound);
    assert!(!serde_json::to_string(&error)?.contains(secret));
    for key in ["selected_model", "post_process_api_keys", "custom_words"] {
        let error = validate_ui(key, json!(secret), &context)
            .err()
            .ok_or_else(|| "protected field accepted".to_owned())?;
        assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
        assert!(!serde_json::to_string(&error)?.contains(secret));
    }
    Ok(())
}

pub(super) fn canonical_cases() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    [
        boolean_cases(),
        enum_cases(),
        number_cases(),
        string_cases(),
    ]
    .concat()
}
fn boolean_cases() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    [
        "audio_feedback",
        "start_hidden",
        "autostart_enabled",
        "update_checks_enabled",
        "show_whats_new_on_update",
        "always_on_microphone",
        "translate_to_english",
        "debug_mode",
        "post_process_enabled",
        "mute_while_recording",
        "append_trailing_space",
        // 015: smart capitalization.
        "smart_capitalization",
        "return_to_dictation_app",
        "experimental_enabled",
        "lazy_stream_close",
        "show_tray_icon",
        "reliable_paste",
        // 007 AC-019: experimental paste verification.
        "paste_verification",
        "filler_word_removal_enabled",
        // 016: smart paragraphs.
        "smart_paragraphs_enabled",
        "vad_enabled",
    ]
    .into_iter()
    // 009: the auto-submit switch is a setting only with the `destination-return` gate closed.
    .chain((!cfg!(feature = "destination-return")).then_some("auto_submit"))
    // 010: the no-text-field bubble setting exists only with the gate open.
    .chain(
        cfg!(feature = "destination-return").then_some("show_recovery_bubble_without_text_field"),
    )
    .map(boolean_case)
    .collect()
}
/// A valid change of `key`: on, except the macOS beta reliable paste once smart paste replaces
/// it (feature 007 rollout gate), which can only stay off.
fn boolean_case(key: &'static str) -> (&'static str, &'static str, serde_json::Value) {
    let replaced = cfg!(all(target_os = "macos", feature = "smart-paste-dispatch"));
    if key == "reliable_paste" && replaced {
        (key, "false", json!(false))
    } else {
        (key, "true", json!(true))
    }
}
fn enum_cases() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    [
        ("shortcut_activation", "hold_or_toggle"),
        ("sound_theme", "pop"),
        ("overlay_position", "top"),
        ("log_level", "warn"),
        ("model_unload_timeout", "sec15"),
        ("recording_retention_period", "months3"),
        ("paste_method", "ctrl_shift_v"),
        ("clipboard_handling", "copy_to_clipboard"),
        ("auto_submit_key", "cmd_enter"),
        ("theme", "dark"),
        ("keyboard_implementation", "handy_keys"),
        ("typing_tool", "ydotool"),
        ("chinese_script", "traditional"),
        ("transcribe_accelerator", "gpu"),
        ("ort_accelerator", "directml"),
        ("vad_backend", "earshot"),
        ("overlay_style", "live"),
    ]
    .into_iter()
    // 009: Automatic Enter replaces it with the `destination-return` gate open.
    .chain(cfg!(feature = "destination-return").then_some(("automatic_enter", "pin_only")))
    .map(|(key, value)| (key, value, json!(value)))
    .collect()
}
fn number_cases() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    vec![
        ("hold_threshold_ms", "151", json!(151)),
        ("audio_feedback_volume", "0.5", json!(0.5)),
        ("selected_channel", "65535", json!(65535)),
        ("word_correction_threshold", "0.75", json!(0.75)),
        ("history_limit", "17", json!(17)),
        ("paste_delay_ms", "11", json!(11)),
        ("paste_delay_after_ms", "13", json!(13)),
        ("extra_recording_buffer_ms", "33", json!(33)),
    ]
}
fn string_cases() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    [
        ("clamshell_microphone", "  Microphone  "),
        ("selected_output_device", "  Output  "),
        ("selected_language", "auto"),
        ("post_process_selected_prompt_id", "known"),
        // 005 AC-009: prompt remembered for plain dictations.
        ("plain_dictation_prompt_id", "known"),
        ("app_language", "fr-FR"),
        ("external_script_path", "  /tmp/script  "),
        ("transcribe_gpu_device", "  opaque selector  "),
    ]
    .into_iter()
    .map(|(key, value)| (key, value, json!(value)))
    .collect()
}
#[test]
fn all_55_scalar_ports_accept_the_same_typed_values() -> Result<(), Box<dyn std::error::Error>> {
    // Every capability, so a platform-bound key is accepted on every build of this test.
    let context = ValidationContext {
        prompt_ids: &["known"],
        updates_allowed: true,
        platform: PlatformCapabilities::only(&[
            PlatformCapability::Destination,
            PlatformCapability::FocusedFieldDetection,
        ]),
    };
    let cases = canonical_cases();
    assert_eq!(
        cases.len(),
        55 + usize::from(cfg!(feature = "destination-return"))
    );
    for (key, raw, value) in cases {
        let cli = validate_cli(key, raw, &context)?;
        let ui = validate_ui(key, value.clone(), &context)?;
        assert_eq!(cli, ui, "{key}");
        assert_eq!(cli.key(), key);
        assert_eq!(serde_json::to_value(cli.value())?, value, "{key}");
        assert!(
            validate_ui(
                key,
                json!({"CANARY_CANDIDATE_SECRET":"not a scalar"}),
                &context
            )
            .is_err(),
            "{key}"
        );
    }
    Ok(())
}
#[test]
fn every_nullable_scalar_preserves_strings_and_accepts_null_clear(
) -> Result<(), Box<dyn std::error::Error>> {
    let context = ValidationContext {
        prompt_ids: &["known"],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    for key in [
        "selected_channel",
        "clamshell_microphone",
        "selected_output_device",
        "post_process_selected_prompt_id",
        "plain_dictation_prompt_id",
        "external_script_path",
        "transcribe_gpu_device",
    ] {
        let cli = validate_cli(key, "null", &context)?;
        let ui = validate_ui(key, json!(null), &context)?;
        assert_eq!(cli, ui);
        assert!(serde_json::to_value(cli.value())?.is_null());
    }
    for key in ["clamshell_microphone", "selected_output_device"] {
        // Null is the only system-default value; "default" is a device name (a real ALSA PCM on
        // Linux) kept byte for byte, so each intention has exactly one stored value.
        for name in [
            "default",
            "Default",
            "DEFAULT",
            "USB Microphone",
            " default",
        ] {
            let cli = validate_cli(key, name, &context)?;
            assert_eq!(validate_ui(key, json!(name), &context)?, cli);
            assert_eq!(serde_json::to_value(cli.value())?, json!(name));
        }
    }
    assert!(validate_ui("transcribe_gpu_device", json!(7), &context).is_err());
    for key in [
        "post_process_selected_prompt_id",
        "plain_dictation_prompt_id",
    ] {
        assert!(
            validate_ui(key, json!("missing"), &context).is_err(),
            "{key}"
        );
    }
    Ok(())
}
#[test]
fn numeric_boundaries_and_overflow_have_identical_safe_error_reasons(
) -> Result<(), Box<dyn std::error::Error>> {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    for (key, min, max) in [
        ("hold_threshold_ms", 100, 1000),
        ("selected_channel", 0, 65535),
        ("history_limit", 0, 1000),
        ("paste_delay_ms", 10, 5000),
        ("paste_delay_after_ms", 10, 5000),
        ("extra_recording_buffer_ms", 0, 1500),
    ] {
        for value in [min, max] {
            assert_eq!(
                validate_cli(key, &value.to_string(), &context)?,
                validate_ui(key, json!(value), &context)?
            );
        }
        let bad = max + 1;
        assert_validation_errors(
            validate_cli(key, &bad.to_string(), &context),
            validate_ui(key, json!(bad), &context),
        )?;
        assert!(validate_cli(key, "18446744073709551616", &context).is_err());
        assert!(validate_cli(key, "-1", &context).is_err());
    }
    for key in ["audio_feedback_volume", "word_correction_threshold"] {
        for value in [0.0, 1.0] {
            assert_eq!(
                validate_cli(key, &value.to_string(), &context)?,
                validate_ui(key, json!(value), &context)?
            );
        }
        for value in [-0.1, 1.1] {
            assert_validation_errors(
                validate_cli(key, &value.to_string(), &context),
                validate_ui(key, json!(value), &context),
            )?;
        }
        for value in ["NaN", "inf", "-inf"] {
            assert!(validate_cli(key, value, &context).is_err());
        }
    }
    Ok(())
}
#[test]
fn legacy_aliases_are_disk_only_and_build_policy_is_shared(
) -> Result<(), Box<dyn std::error::Error>> {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: PlatformCapabilities::current(),
    };
    for (key, raw, value) in [
        ("log_level", "WARN", json!("WARN")),
        ("log_level", "3", json!(3)),
        ("overlay_position", "none", json!("none")),
    ] {
        assert!(validate_cli(key, raw, &context).is_err());
        assert!(validate_ui(key, value, &context).is_err());
    }
    let blocked = ValidationContext {
        prompt_ids: &[],
        updates_allowed: false,
        platform: PlatformCapabilities::current(),
    };
    for (value, text) in [(true, "true"), (false, "false")] {
        assert_validation_errors(
            validate_cli("update_checks_enabled", text, &blocked),
            validate_ui("update_checks_enabled", json!(value), &blocked),
        )?;
    }
    Ok(())
}
fn assert_validation_errors(
    cli: Result<SettingsMutation, SettingsError>,
    ui: Result<SettingsMutation, SettingsError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli
        .err()
        .ok_or_else(|| "CLI accepted invalid value".to_owned())?;
    let ui = ui
        .err()
        .ok_or_else(|| "UI accepted invalid value".to_owned())?;
    assert_eq!(cli.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(ui.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(cli, ui);
    Ok(())
}
