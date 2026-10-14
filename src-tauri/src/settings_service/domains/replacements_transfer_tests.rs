//! Private transfer steps of the replacements domain: byte-bounded pages, and an import staged
//! in chunks that commits exactly like a direct import.
use super::{offline, outcome, pairs, rules, texts, Call, TestResult};
use crate::word_replacements::transfer::sha256_hex;
use serde_json::{json, Value};

fn large_input(count: usize) -> String {
    (0..count)
        .map(|index| format!("{}{index};{}\n", "字".repeat(60), "😀".repeat(150)))
        .collect()
}

#[test]
fn pages_are_byte_bounded_and_cover_the_list_in_order() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("import", &["replace"])
        .input(&large_input(1000))
        .run(&service);
    let (mut collected, mut start) = (Vec::new(), 0);
    loop {
        let start_text = start.to_string();
        let (code, _, data) = outcome(&Call::new("read-page", &[&start_text]).run(&service));
        assert_eq!((code, &data["total"]), (0, &json!(1000)));
        assert!(data.to_string().len() <= super::super::MAX_PAGE_BYTES + 1024);
        collected.extend(data["rules"].as_array().cloned().unwrap_or_default());
        match data["next"].as_u64() {
            Some(next) => start = next as usize,
            None => break,
        }
    }
    assert!(start > 0, "a large list spans several pages");
    assert_eq!(Value::Array(collected), rules(&service));
    Ok(())
}

#[test]
fn a_staged_import_commits_like_a_direct_one_and_only_once() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let input = "recognized;replacement\ncloud code;Claude Code\nget hub;GitHub\n";
    let length = input.len().to_string();
    let begun = Call::new("stage-begin", &[&length, &sha256_hex(input)]).run(&service);
    let token = outcome(&begun).2["token"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let (head, tail) = input.split_at(20);
    Call::new("stage-append", &[&token, "0"])
        .input(head)
        .run(&service);
    Call::new("stage-append", &[&token, "1"])
        .input(tail)
        .run(&service);
    let (code, revision, data) = outcome(&Call::new("import", &["merge", &token]).run(&service));
    assert_eq!((code, revision, &data["added"]), (0, json!(1), &json!(2)));
    let expected = pairs(&[("cloud code", "Claude Code"), ("get hub", "GitHub")]);
    assert_eq!(texts(&rules(&service)), expected);
    let again = Call::new("import", &["merge", &token]).run(&service);
    assert_eq!(
        outcome(&again).2["code"],
        "unavailable",
        "a token is taken once"
    );
    Ok(())
}

#[test]
fn hostile_staging_steps_are_refused_with_stable_codes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let code = |call: Call<'_>| outcome(&call.run(&service)).2["code"].clone();
    assert_eq!(
        code(Call::new("stage-append", &["forged", "0"]).input("x")),
        "unavailable"
    );
    assert_eq!(
        code(Call::new("import", &["merge", "forged"])),
        "unavailable"
    );
    assert_eq!(
        code(Call::new("stage-begin", &["2000000", "00"])),
        "limit_exceeded"
    );
    let begun = Call::new("stage-begin", &["3", &sha256_hex("abc")]).run(&service);
    let token = outcome(&begun).2["token"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert_eq!(
        code(Call::new("stage-append", &[&token, "1"]).input("c")),
        "usage_error"
    );
    assert_eq!(code(Call::new("import", &["merge", &token])), "usage_error");
    assert_eq!(code(Call::new("read-page", &["first"])), "usage_error");
    assert!(rules(&service).as_array().is_some_and(Vec::is_empty));
    Ok(())
}

#[test]
fn read_settings_is_the_public_get_without_the_rules() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["cloud code", "Claude Code"]).run(&service);
    let (code, _, data) = outcome(&Call::new("read-settings", &[]).run(&service));
    let mut public = serde_json::to_value(service.snapshot().public_settings())?;
    let removed = public
        .as_object_mut()
        .and_then(|object| object.remove(super::super::RULES_KEY));
    assert_eq!(
        removed.map(|rules| rules.as_array().map(Vec::len)),
        Some(Some(1))
    );
    assert_eq!((code, data), (0, public));
    Ok(())
}
