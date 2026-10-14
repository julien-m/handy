//! Rule files of the settings window: bounded UTF-8 import reads with typed refusals that name the
//! chosen file, and atomic canonical exports of the saved rules.
use super::{read_import_file, write_export_file};
use crate::settings_service::errors::{ErrorReason, SettingsErrorCode, ValidationReason};
use crate::word_replacements::model::{ReplacementRule, RuleId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rule(id: &str, recognized: &str, replacement: &str) -> ReplacementRule {
    ReplacementRule {
        id: RuleId::from(id),
        recognized: recognized.to_owned(),
        replacement: replacement.to_owned(),
    }
}

#[test]
fn a_utf8_file_is_read_with_its_name() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rules-team.csv");
    std::fs::write(
        &path,
        "\u{FEFF}recognized;replacement\r\nCloud Code;Claude Code\r\n",
    )?;
    let file = read_import_file(&path).map_err(|error| error.error)?;
    assert_eq!(file.file_name, "rules-team.csv");
    // The text is returned unchanged: the shared parser removes the BOM and the CR.
    assert_eq!(
        file.text,
        "\u{FEFF}recognized;replacement\r\nCloud Code;Claude Code\r\n"
    );
    Ok(())
}

#[test]
fn a_file_that_is_not_utf8_is_refused_with_its_name() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.rtf");
    std::fs::write(&path, [b'a', b';', 0xFF, b'b'])?;
    let refused = read_import_file(&path).err().ok_or("accepted")?;
    assert_eq!(refused.file_name, "notes.rtf");
    assert_eq!(refused.error.code, SettingsErrorCode::ValidationFailed);
    assert_eq!(
        refused.error.reason,
        Some(ValidationReason::InvalidEncoding)
    );
    Ok(())
}

#[test]
fn the_import_cap_is_inclusive() -> TestResult {
    let directory = tempfile::tempdir()?;
    let at_cap = directory.path().join("at-cap.txt");
    std::fs::write(&at_cap, vec![b'\n'; 1_000_000])?;
    assert!(read_import_file(&at_cap).is_ok());
    let above = directory.path().join("above.txt");
    std::fs::write(&above, vec![b'\n'; 1_000_001])?;
    let refused = read_import_file(&above).err().ok_or("accepted")?;
    assert_eq!(refused.file_name, "above.txt");
    assert_eq!(refused.error.code, SettingsErrorCode::LimitExceeded);
    Ok(())
}

#[test]
fn a_missing_file_is_unreadable() -> TestResult {
    let directory = tempfile::tempdir()?;
    let refused = read_import_file(&directory.path().join("gone.txt"))
        .err()
        .ok_or("accepted")?;
    assert_eq!(refused.file_name, "gone.txt");
    assert_eq!(refused.error.code, SettingsErrorCode::UsageError);
    Ok(())
}

#[test]
fn export_writes_the_canonical_rules_in_order() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("replacement-rules.txt");
    let rules = [
        rule("a", "Cloud Code", "Claude Code"),
        rule("b", "Open Claw", "OpenClaw;CLI"),
    ];
    let written = write_export_file(&path, &rules).map_err(|error| error.error)?;
    assert_eq!(written.file_name, "replacement-rules.txt");
    assert_eq!(written.count, 2);
    assert_eq!(
        std::fs::read_to_string(&path)?,
        "recognized;replacement\nCloud Code;Claude Code\nOpen Claw;OpenClaw;CLI\n"
    );
    // The exported file reads back as an import source.
    assert!(read_import_file(&path).is_ok());
    Ok(())
}

#[cfg(unix)]
#[test]
fn export_refuses_a_symbolic_link_and_names_the_file() -> TestResult {
    let directory = tempfile::tempdir()?;
    let target = directory.path().join("target.txt");
    std::fs::write(&target, "kept")?;
    let link = directory.path().join("link.txt");
    std::os::unix::fs::symlink(&target, &link)?;
    let refused = write_export_file(&link, &[rule("a", "x", "y")])
        .err()
        .ok_or("written")?;
    assert_eq!(refused.file_name, "link.txt");
    assert_eq!(refused.error.code, SettingsErrorCode::PersistenceFailed);
    assert_eq!(
        refused.error.detail_reason,
        Some(ErrorReason::ExportSymbolicLink)
    );
    assert_eq!(std::fs::read_to_string(&target)?, "kept");
    Ok(())
}
