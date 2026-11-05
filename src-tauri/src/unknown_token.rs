//! Restores French quotation marks that a model writes as its unknown token.
//!
//! Canary has no « or » in its vocabulary and writes the literal token `<unk>` in their place
//! (« répondre <unk> Ok <unk>. »). Decision `002-word-replacements:AC-007` /
//! `unknown_token_quotes` (Julien, 2026-10-30): consecutive tokens separated only by spaces count
//! as one, the remaining tokens alternate « and », and an unpaired last token is removed. The
//! spacing the model wrote around each token is kept.

// @spec AC-007: Replacement pipeline position — .specs/features/002-word-replacements/spec.md#ac-007

/// The unknown token written by Canary.
const UNKNOWN_TOKEN: &str = "<unk>";
/// Punctuation that sticks to the preceding word, so no space is inserted before it.
const CLOSING_PUNCTUATION: [char; 8] = ['.', ',', ';', ':', '!', '?', ')', ']'];

/// Replaces paired unknown tokens with « and », collapsing repeated tokens and dropping an
/// unpaired last one. Text without the token is returned unchanged.
pub fn restore_quotes(text: &str) -> String {
    if !text.contains(UNKNOWN_TOKEN) {
        return text.to_owned();
    }
    let segments = collapse_repeats(text.split(UNKNOWN_TOKEN).collect());
    let markers = segments.len() - 1;
    let paired = markers - markers % 2;
    let mut out = String::from(segments[0]);
    for (index, segment) in segments[1..].iter().enumerate() {
        if index < paired {
            out.push(if index % 2 == 0 { '«' } else { '»' });
            out.push_str(segment);
        } else {
            join_without_token(&mut out, segment);
        }
    }
    out
}

/// Merges tokens separated only by whitespace: the whitespace between them is dropped.
fn collapse_repeats(segments: Vec<&str>) -> Vec<&str> {
    let last = segments.len() - 1;
    segments
        .iter()
        .enumerate()
        .filter(|(index, segment)| *index == 0 || *index == last || !segment.trim().is_empty())
        .map(|(_, segment)| *segment)
        .collect()
}

/// Joins the text after a removed token, leaving one space between words and none before
/// closing punctuation.
fn join_without_token(out: &mut String, right: &str) {
    let right = right.trim_start();
    out.truncate(out.trim_end().len());
    let needs_space = !out.is_empty()
        && right
            .chars()
            .next()
            .is_some_and(|first| !CLOSING_PUNCTUATION.contains(&first));
    if needs_space {
        out.push(' ');
    }
    out.push_str(right);
}

#[cfg(test)]
mod tests {
    use super::restore_quotes;

    // Transcripts of Julien's sessions (2026-10-16 to 2026-10-30) where Canary wrote `<unk>`.
    #[test]
    fn paired_tokens_become_french_quotes() {
        assert_eq!(
            restore_quotes("répondre <unk> Ok <unk>."),
            "répondre « Ok »."
        );
        assert_eq!(
            restore_quotes("la session <unk> Clarifier le pivot de Livia <unk>, dans lequel"),
            "la session « Clarifier le pivot de Livia », dans lequel"
        );
        assert_eq!(
            restore_quotes("le <unk> readme <unk> doit être en anglais"),
            "le « readme » doit être en anglais"
        );
    }

    #[test]
    fn repeated_tokens_count_as_one() {
        assert_eq!(
            restore_quotes("tous les <unk> <unk> readme <unk> sont en anglais"),
            "tous les « readme » sont en anglais"
        );
        assert_eq!(restore_quotes("<unk><unk>Ok<unk>"), "«Ok»");
    }

    #[test]
    fn an_unpaired_last_token_is_removed() {
        assert_eq!(
            restore_quotes("le <unk> readme <unk> et <unk> la suite"),
            "le « readme » et la suite"
        );
        assert_eq!(restore_quotes("fin du texte <unk>."), "fin du texte.");
        assert_eq!(restore_quotes("<unk> seul"), "seul");
        assert_eq!(restore_quotes("<unk>"), "");
    }

    #[test]
    fn text_without_the_token_is_unchanged() {
        let text = "un texte <sans> jeton";
        assert_eq!(restore_quotes(text), text);
    }
}
