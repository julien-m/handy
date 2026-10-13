//! Historical CLI parsing and GUI/headless console policy regressions.

use super::{cli::CliArgs, console_windows};
use clap::Parser;
use std::path::PathBuf;

#[test]
fn historical_boolean_flags_keep_their_parser_values() -> Result<(), clap::Error> {
    type BooleanFlag = (&'static str, fn(&CliArgs) -> bool);
    let flags: [BooleanFlag; 9] = [
        ("--start-hidden", |args| args.start_hidden),
        ("--no-tray", |args| args.no_tray),
        ("--toggle-transcription", |args| args.toggle_transcription),
        ("--toggle-post-process", |args| args.toggle_post_process),
        ("--cancel", |args| args.cancel),
        ("--debug", |args| args.debug),
        ("--list-devices", |args| args.list_devices),
        ("--list-models", |args| args.list_models),
        ("--json", |args| args.json),
    ];
    let default = CliArgs::try_parse_from(["handy"])?;
    assert!(flags.iter().all(|(_, value)| !value(&default)));
    assert!(default.transcribe_file.is_none());
    assert!(default.model.is_none());
    assert!(default.device_index.is_none());
    assert!(default.repeat.is_none());

    for (flag, _) in flags {
        let parsed = CliArgs::try_parse_from(["handy", flag])?;
        for (other_flag, value) in flags {
            assert_eq!(value(&parsed), flag == other_flag, "{flag}: {other_flag}");
        }
    }
    Ok(())
}

#[test]
fn historical_value_flags_keep_values_and_transcription_combinations() -> Result<(), clap::Error> {
    for flag in ["--transcribe-file", "-f"] {
        let parsed = CliArgs::try_parse_from(["handy", flag, "recording sample.wav"])?;
        assert_eq!(
            parsed.transcribe_file,
            Some(PathBuf::from("recording sample.wav"))
        );
    }
    assert_eq!(
        CliArgs::try_parse_from(["handy", "--model", "whisper-small"])?
            .model
            .as_deref(),
        Some("whisper-small")
    );
    assert_eq!(
        CliArgs::try_parse_from(["handy", "--device-index", "0"])?.device_index,
        Some(0)
    );
    assert_eq!(
        CliArgs::try_parse_from(["handy", "--repeat", "3"])?.repeat,
        Some(3)
    );

    let parsed = CliArgs::try_parse_from([
        "handy",
        "--transcribe-file",
        "recording sample.wav",
        "--model",
        "whisper-small",
        "--device-index",
        "2",
        "--repeat",
        "3",
        "--json",
        "--debug",
    ])?;
    assert_eq!(
        parsed.transcribe_file,
        Some(PathBuf::from("recording sample.wav"))
    );
    assert_eq!(parsed.model.as_deref(), Some("whisper-small"));
    assert_eq!(parsed.device_index, Some(2));
    assert_eq!(parsed.repeat, Some(3));
    assert!(parsed.json && parsed.debug);
    Ok(())
}

#[test]
fn historical_startup_and_recording_flags_remain_combinable() -> Result<(), clap::Error> {
    let parsed = CliArgs::try_parse_from([
        "handy",
        "--start-hidden",
        "--no-tray",
        "--toggle-transcription",
        "--toggle-post-process",
        "--cancel",
        "--debug",
        "--json",
        "--list-devices",
        "--list-models",
    ])?;
    assert!(parsed.start_hidden && parsed.no_tray);
    assert!(parsed.toggle_transcription && parsed.toggle_post_process && parsed.cancel);
    assert!(parsed.debug && parsed.json && parsed.list_devices && parsed.list_models);
    Ok(())
}

#[test]
fn parsed_gui_detaches_console_and_headless_keeps_it() -> Result<(), clap::Error> {
    for flags in [
        vec![],
        vec!["--start-hidden"],
        vec!["--no-tray"],
        vec!["--toggle-transcription"],
        vec!["--toggle-post-process"],
        vec!["--cancel"],
        vec!["--debug"],
        vec!["--json"],
        vec!["--model", "whisper-small"],
        vec!["--device-index", "0"],
        vec!["--repeat", "3"],
        vec![
            "--start-hidden",
            "--no-tray",
            "--toggle-transcription",
            "--debug",
        ],
    ] {
        let parsed =
            CliArgs::try_parse_from(std::iter::once("handy").chain(flags.iter().copied()))?;
        assert!(parsed.should_detach_console(), "{flags:?}");
    }
    for flags in [
        vec!["--transcribe-file", "sample.wav"],
        vec!["-f", "sample.wav"],
        vec!["--list-devices"],
        vec!["--list-models"],
        vec!["--list-models", "--json", "--debug"],
        vec!["--list-devices", "--start-hidden", "--no-tray"],
        vec![
            "--transcribe-file",
            "sample.wav",
            "--model",
            "whisper-small",
            "--repeat",
            "3",
            "--json",
        ],
    ] {
        let parsed =
            CliArgs::try_parse_from(std::iter::once("handy").chain(flags.iter().copied()))?;
        assert!(!parsed.should_detach_console(), "{flags:?}");
    }
    Ok(())
}

#[test]
fn detach_success_including_no_attached_console_is_accepted() -> std::io::Result<()> {
    let mut calls = 0;
    console_windows::detach_console_with(|| {
        calls += 1;
        Ok(())
    })?;
    assert_eq!(calls, 1);
    Ok(())
}

#[test]
fn detach_failure_retains_error_kind_and_safe_operation_context() {
    let error = console_windows::detach_console_with(|| {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    })
    .expect_err("native detach failure must propagate");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(error.to_string().contains("detach GUI process console"));
}

#[test]
fn historical_invocations_parse_without_a_subcommand() -> Result<(), clap::Error> {
    for args in [
        vec!["handy"],
        vec!["handy", "--start-hidden", "--no-tray"],
        vec!["handy", "--list-models", "--json"],
        vec!["handy", "--transcribe-file", "settings", "--json"],
        vec!["handy", "--model", "settings", "--debug"],
    ] {
        assert!(
            CliArgs::try_parse_from(&args)?.command.is_none(),
            "{args:?}"
        );
    }
    Ok(())
}

fn settings_of(args: &[&str]) -> Result<(CliArgs, super::cli::SettingsArgs), clap::Error> {
    let parsed = CliArgs::try_parse_from(args)?;
    match parsed.command.clone() {
        Some(super::cli::CliCommand::Settings(settings)) => Ok((parsed, settings)),
        None => Err(clap::Error::new(clap::error::ErrorKind::MissingSubcommand)),
    }
}

#[test]
fn settings_set_takes_a_negative_value_and_a_leading_json_flag() -> Result<(), clap::Error> {
    use super::cli::SettingsCommand;
    let args = ["handy", "--json", "settings", "set", "paste_delay_ms", "-5"];
    let (parsed, settings) = settings_of(&args)?;
    assert!(parsed.json || settings.json);
    let SettingsCommand::Set { key, value, .. } = settings.operation else {
        return Err(clap::Error::new(clap::error::ErrorKind::InvalidSubcommand));
    };
    assert_eq!((key.as_str(), value.as_str()), ("paste_delay_ms", "-5"));
    Ok(())
}

#[test]
fn settings_set_takes_an_expected_revision_and_a_trailing_json_flag() -> Result<(), clap::Error> {
    use super::cli::SettingsCommand;
    let args = [
        "handy",
        "settings",
        "set",
        "k",
        "v",
        "--expected-revision",
        "4",
        "--json",
    ];
    let (parsed, settings) = settings_of(&args)?;
    assert!(settings.json || parsed.json);
    assert!(matches!(
        settings.operation,
        SettingsCommand::Set {
            expected_revision: Some(4),
            ..
        }
    ));
    Ok(())
}
