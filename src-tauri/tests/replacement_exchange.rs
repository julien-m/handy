//! Exchange-format integration tests against real temporary files (AC-004/AC-005, SC-002).

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use handy_app_lib::word_replacements::{
    exchange::{read_exchange_input, serialize_exchange, ExchangeInputError, MAX_IMPORT_BYTES},
    model::{ReplacementRule, RuleId, RuleIdSource},
    preview::{materialize_candidate, preview_exchange, ImportMode, PreviewStatus},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct SequentialIds(usize);

impl RuleIdSource for SequentialIds {
    fn next_id(&mut self) -> RuleId {
        self.0 += 1;
        RuleId::from(format!("id-{}", self.0))
    }
}

fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    let path = dir.join(name);
    File::create(&path)?.write_all(bytes)?;
    Ok(path)
}

fn read_file(path: &Path) -> std::io::Result<Result<String, ExchangeInputError>> {
    Ok(read_exchange_input(File::open(path)?))
}

fn sample_rules(count: usize) -> Vec<ReplacementRule> {
    (0..count)
        .map(|index| ReplacementRule {
            id: RuleId::from(format!("saved-{index}")),
            recognized: format!("Règle {index} 東京"),
            replacement: format!("Remplacement {index}; ok #{index}"),
        })
        .collect()
}

fn content(rules: &[ReplacementRule]) -> Vec<(&str, &str)> {
    rules
        .iter()
        .map(|rule| (rule.recognized.as_str(), rule.replacement.as_str()))
        .collect()
}

/// Build a valid exchange file of exactly `size` bytes: one rule plus a comment pad.
fn exact_size_exchange(size: usize) -> Vec<u8> {
    let mut bytes = b"Cloud Code;Claude Code\n#".to_vec();
    bytes.resize(size, b'x');
    bytes
}

#[test]
fn accept_files_up_to_the_cap_and_reject_one_byte_more() -> TestResult {
    let dir = tempfile::tempdir()?;
    for (size, is_accepted) in [
        (MAX_IMPORT_BYTES - 1, true),
        (MAX_IMPORT_BYTES, true),
        (MAX_IMPORT_BYTES + 1, false),
    ] {
        let path = write_file(
            dir.path(),
            &format!("rules-{size}.txt"),
            &exact_size_exchange(size),
        )?;
        let result = read_file(&path)?;
        assert_eq!(result.is_ok(), is_accepted, "size {size}");
        if !is_accepted {
            let expected = ExchangeInputError::TooLarge {
                max_bytes: MAX_IMPORT_BYTES,
            };
            assert_eq!(result, Err(expected));
        }
    }
    Ok(())
}

#[test]
fn reject_invalid_utf8_file_with_byte_offset() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = write_file(
        dir.path(),
        "latin1.txt",
        b"Cloud Code;Claude Code\nCaf\xE9;Coffee\n",
    )?;
    assert_eq!(
        read_file(&path)?,
        Err(ExchangeInputError::InvalidUtf8 { byte_offset: 26 })
    );
    Ok(())
}

#[test]
fn import_windows_file_with_bom_and_crlf() -> TestResult {
    let dir = tempfile::tempdir()?;
    let source =
        "\u{FEFF}texte_reconnu;remplacement\r\nCloud Code;Claude Code\r\nOpen Claw\tOpenClaw\r\n";
    let path = write_file(dir.path(), "windows.txt", source.as_bytes())?;
    let preview = preview_exchange(&[], &read_file(&path)??, ImportMode::Replace);
    let statuses: Vec<_> = preview.lines.iter().map(|line| line.status).collect();
    let expected = [
        PreviewStatus::Ignored,
        PreviewStatus::Add,
        PreviewStatus::Add,
    ];
    assert_eq!(statuses, expected);
    let rules = materialize_candidate(&preview, &mut SequentialIds(0))?;
    let expected = [("Cloud Code", "Claude Code"), ("Open Claw", "OpenClaw")];
    assert_eq!(content(&rules), expected);
    Ok(())
}

#[test]
fn export_then_replace_import_reproduces_1_100_and_1000_rules() -> TestResult {
    let dir = tempfile::tempdir()?;
    let unrelated = vec![ReplacementRule {
        id: RuleId::from("other"),
        recognized: "Unrelated".into(),
        replacement: "Gone".into(),
    }];
    for count in [1, 100, 1000] {
        let exported = sample_rules(count);
        let path = write_file(
            dir.path(),
            &format!("export-{count}.txt"),
            serialize_exchange(&exported).as_bytes(),
        )?;
        let text = read_file(&path)??;
        // Re-import onto a different set: content and order equal the export.
        let preview = preview_exchange(&unrelated, &text, ImportMode::Replace);
        let imported = materialize_candidate(&preview, &mut SequentialIds(0))?;
        assert_eq!(content(&imported), content(&exported), "count {count}");
        // Re-import onto the same set is a no-op that keeps every identity.
        let same = preview_exchange(&exported, &text, ImportMode::Replace);
        assert!(same.is_noop, "count {count}");
        assert_eq!(
            materialize_candidate(&same, &mut SequentialIds(0))?,
            exported
        );
    }
    Ok(())
}

#[test]
fn merge_import_file_updates_in_place_and_appends() -> TestResult {
    let dir = tempfile::tempdir()?;
    let base = vec![ReplacementRule {
        id: RuleId::from("kept"),
        recognized: "Cloud Code".into(),
        replacement: "Claude Code".into(),
    }];
    let source = b"cloud code;Claude Code CLI\nOpen Claw;OpenClaw\n";
    let path = write_file(dir.path(), "merge.txt", source)?;
    let preview = preview_exchange(&base, &read_file(&path)??, ImportMode::Merge);
    let rules = materialize_candidate(&preview, &mut SequentialIds(0))?;
    let expected = [("Cloud Code", "Claude Code CLI"), ("Open Claw", "OpenClaw")];
    assert_eq!(content(&rules), expected);
    assert_eq!(rules[0].id, RuleId::from("kept"));
    Ok(())
}
