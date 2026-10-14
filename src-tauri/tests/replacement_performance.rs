//! Release-build matching budget: 1000 rules over a 2000-scalar transcript (spec QE).
//!
//! Run with `cargo test --release --test replacement_performance -- --nocapture`.
//! Debug builds skip it because unoptimized timings say nothing about the budget.

use std::time::{Duration, Instant};

use handy_app_lib::word_replacements::{
    dictation::compile_for_dictation,
    model::{ReplacementRule, RuleId},
};

/// Spec QE budget per dictation, stated for an Apple M1 release build.
const MATCHING_BUDGET: Duration = Duration::from_millis(20);
const RULE_COUNT: usize = 1000;
const TRANSCRIPT_SCALARS: usize = 2000;
const SAMPLES: usize = 30;

/// Adversarial rules: long shared prefixes, folding expansions and CJK keys,
/// so the trie walk at most positions goes deep before failing.
fn adversarial_rules() -> Vec<ReplacementRule> {
    (0..RULE_COUNT)
        .map(|index| {
            let recognized = match index % 3 {
                0 => format!("alpha beta gamma delta {index}"),
                1 => format!("STRASSE straße ﬁle {index}"),
                _ => format!("東京クロード{index}"),
            };
            ReplacementRule {
                id: RuleId::from(format!("rule-{index}")),
                recognized,
                replacement: format!("Replacement {index}"),
            }
        })
        .collect()
}

/// Near-miss transcript: every rule prefix appears but most never complete.
fn adversarial_transcript() -> String {
    let pattern = "alpha beta gamma delta x straße ﬁle y 東京クロードz cloud code, ";
    pattern.chars().cycle().take(TRANSCRIPT_SCALARS).collect()
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only timing budget")]
fn match_1000_rules_over_2000_scalars_within_budget() {
    let rules = adversarial_rules();
    let transcript = adversarial_transcript();
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        // Measure compile + apply: until 008 publishes a compiled snapshot,
        // each dictation compiles its captured rules once.
        let started = Instant::now();
        let output = compile_for_dictation(&rules).apply(&transcript);
        samples.push(started.elapsed());
        assert!(!output.text().is_empty());
    }
    let maximum = samples.iter().max().copied().unwrap_or_default();
    println!(
        "replacement_performance rules={RULE_COUNT} scalars={TRANSCRIPT_SCALARS} samples_us={:?} max_us={}",
        samples.iter().map(Duration::as_micros).collect::<Vec<_>>(),
        maximum.as_micros()
    );
    assert!(maximum < MATCHING_BUDGET, "slowest sample {maximum:?}");
}
