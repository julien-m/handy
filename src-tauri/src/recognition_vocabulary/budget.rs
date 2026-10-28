//! Context budgets and whole-term fitting. The engine worker counts tokens with the loaded
//! model's own tokenizer, so the prompt Handy sends is never trimmed again by the engine and
//! every term reported as sent reaches it whole.
use serde::{Deserialize, Serialize};

/// Whisper initial-prompt budget in tokens: `n_text_ctx / 2 - 1` with `n_text_ctx = 448`, the
/// prompt cap of the pinned transcribe-cpp 0.3.1 (`whisper_prev_cap`) and of the OpenAI
/// reference decoder (see `contracts/context-budget.md`).
pub const WHISPER_PROMPT_BUDGET_TOKENS: u32 = 223;

/// Separator of the rendered term list (the existing Whisper initial prompt format).
pub const TERM_SEPARATOR: &str = ", ";

/// Unit of a context budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum BudgetUnit {
    Tokens,
}

/// Documented context budget of one mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ContextBudget {
    pub value: u32,
    pub unit: BudgetUnit,
}

impl ContextBudget {
    /// The Whisper initial-prompt budget.
    pub const WHISPER: Self = Self {
        value: WHISPER_PROMPT_BUDGET_TOKENS,
        unit: BudgetUnit::Tokens,
    };
}

/// Token counting of the loaded model (the worker's tokenizer; a fake in tests).
pub trait TokenCounter {
    /// Number of tokens of `text`.
    ///
    /// # Errors
    /// A tokenizer failure, described without the text.
    fn count_tokens(&self, text: &str) -> Result<usize, String>;
}

/// Token counter over a loaded transcribe-cpp model: the tokenizer the engine itself uses for
/// the prompt (worker side).
pub struct ModelTokenizer(pub transcribe_cpp::Model);
impl TokenCounter for ModelTokenizer {
    fn count_tokens(&self, text: &str) -> Result<usize, String> {
        self.0
            .tokenize(text)
            .map(|ids| ids.len())
            .map_err(|error| format!("tokenizer failed: {error}"))
    }
}

/// The initial prompt of `terms`, as Handy sends it.
pub fn render_terms(terms: &[String]) -> String {
    terms.join(TERM_SEPARATOR)
}

/// Text the Whisper engine tokenizes for `prompt`: it strips the prompt and prepends one space
/// (transcribe-cpp `whisper_run`, the Hugging Face `get_prompt_ids` form).
fn whisper_token_text(prompt: &str) -> String {
    format!(" {}", prompt.trim())
}

/// Tokens of the Whisper prompt made of the first `count` terms.
fn prompt_tokens(
    terms: &[String],
    count: usize,
    counter: &dyn TokenCounter,
) -> Result<usize, String> {
    counter.count_tokens(&whisper_token_text(&render_terms(&terms[..count])))
}

/// Result of fitting terms into a budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FittedPrefix {
    /// Number of leading terms sent (a whole prefix of the list).
    pub sent: usize,
    /// Tokens of the rendered prompt of those terms (0 when none is sent).
    pub tokens: usize,
}

/// The longest whole prefix of `terms` whose rendered Whisper prompt fits `budget` tokens:
/// terms are taken in list order and the first one that does not fit stops the list, so every
/// later term is reported as not sent even if a shorter one would fit.
///
/// # Errors
/// A tokenizer failure.
// @spec FR-003: Whole terms in list order within budget — .specs/features/003-recognition-vocabulary/spec.md#fr-003
pub fn fit_whisper_prefix(
    terms: &[String],
    budget: u32,
    counter: &dyn TokenCounter,
) -> Result<FittedPrefix, String> {
    let budget = budget as usize;
    let mut fitted = FittedPrefix { sent: 0, tokens: 0 };
    if terms.is_empty() {
        return Ok(fitted);
    }
    // Common case first: the whole list fits in one tokenizer call.
    let tokens = prompt_tokens(terms, terms.len(), counter)?;
    if tokens <= budget {
        let sent = terms.len();
        return Ok(FittedPrefix { sent, tokens });
    }
    for count in 1..terms.len() {
        let tokens = prompt_tokens(terms, count, counter)?;
        // Stop at the first term that overflows: the sent list stays an ordered prefix.
        if tokens > budget {
            break;
        }
        fitted = FittedPrefix {
            sent: count,
            tokens,
        };
    }
    Ok(fitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One token per character, so budgets are easy to reason about.
    struct CharCounter;
    impl TokenCounter for CharCounter {
        fn count_tokens(&self, text: &str) -> Result<usize, String> {
            Ok(text.chars().count())
        }
    }
    struct FailingCounter;
    impl TokenCounter for FailingCounter {
        fn count_tokens(&self, _text: &str) -> Result<usize, String> {
            Err("tokenizer has no encode path".into())
        }
    }
    fn terms(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn whole_list_within_budget_is_sent() {
        let list = terms(&["Claude Code", "OpenClaw"]);
        // " Claude Code, OpenClaw" = 22 characters.
        let fitted = fit_whisper_prefix(&list, 22, &CharCounter);
        assert_eq!(
            fitted,
            Ok(FittedPrefix {
                sent: 2,
                tokens: 22
            })
        );
    }

    #[test]
    fn first_overflowing_term_stops_the_prefix() {
        let list = terms(&["abc", "a-very-long-term", "d"]);
        // " abc" = 4, " abc, a-very-long-term" = 22, " abc, a-very-long-term, d" = 25.
        let fitted = fit_whisper_prefix(&list, 10, &CharCounter);
        assert_eq!(fitted, Ok(FittedPrefix { sent: 1, tokens: 4 }));
    }

    #[test]
    fn budget_boundary_is_inclusive() {
        let list = terms(&["ab", "cd"]);
        // " ab, cd" = 7.
        assert_eq!(
            fit_whisper_prefix(&list, 7, &CharCounter).map(|f| f.sent),
            Ok(2)
        );
        assert_eq!(
            fit_whisper_prefix(&list, 6, &CharCounter).map(|f| f.sent),
            Ok(1)
        );
        assert_eq!(
            fit_whisper_prefix(&list, 2, &CharCounter).map(|f| f.sent),
            Ok(0)
        );
    }

    #[test]
    fn whisper_budget_is_the_documented_223_tokens() {
        // 223 one-character terms render to 223 + 2 * 222 = 667 characters; with one-token
        // terms and free separators the budget allows exactly 223 terms.
        struct WordCounter;
        impl TokenCounter for WordCounter {
            fn count_tokens(&self, text: &str) -> Result<usize, String> {
                Ok(text.split(TERM_SEPARATOR).count())
            }
        }
        let list = (0..300)
            .map(|index| format!("t{index}"))
            .collect::<Vec<_>>();
        let fitted = fit_whisper_prefix(&list, WHISPER_PROMPT_BUDGET_TOKENS, &WordCounter);
        assert_eq!(fitted.map(|f| f.sent), Ok(223));
        assert_eq!(ContextBudget::WHISPER.value, 223);
    }

    #[test]
    fn empty_list_needs_no_tokenizer_and_failures_propagate() {
        assert_eq!(
            fit_whisper_prefix(&[], 10, &FailingCounter),
            Ok(FittedPrefix { sent: 0, tokens: 0 })
        );
        assert!(fit_whisper_prefix(&terms(&["a"]), 10, &FailingCounter).is_err());
    }
}
