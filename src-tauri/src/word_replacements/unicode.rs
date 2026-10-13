//! Pinned Unicode 17 folding/properties and original UTF-8 source boundaries.

use std::{collections::HashMap, sync::OnceLock};

/// Unicode version shared by folding, whitespace, category and script tables.
pub const UNICODE_VERSION: &str = "17.0.0";

/// Embedded tables were malformed; consumers must reject preparation safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeDataError;

impl std::fmt::Display for UnicodeDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Invalid embedded Unicode replacement data")
    }
}

impl std::error::Error for UnicodeDataError {}

/// One folded scalar and its complete originating scalar or whitespace run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldedToken {
    pub character: char,
    pub source_start: usize,
    pub source_end: usize,
    pub is_source_start: bool,
    pub is_source_end: bool,
}

/// Validated immutable data loaded once; matching itself remains infallible.
#[derive(Debug, Default)]
pub struct UnicodeData {
    folds: HashMap<char, Vec<char>>,
    whitespace: Vec<(u32, u32)>,
    letters_digits: Vec<(u32, u32)>,
    marks: Vec<(u32, u32)>,
    cjk: Vec<(u32, u32)>,
}

static DATA: OnceLock<Result<UnicodeData, UnicodeDataError>> = OnceLock::new();

impl UnicodeData {
    /// Decode checksum-pinned embedded tables once, returning a stable failure.
    pub fn load() -> Result<&'static Self, UnicodeDataError> {
        DATA.get_or_init(|| Self::parse(include_str!("unicode-data.txt")))
            .as_ref()
            .map_err(|error| *error)
    }

    fn parse(text: &str) -> Result<Self, UnicodeDataError> {
        let mut data = Self::default();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            let fields: Vec<_> = line.split(';').collect();
            if fields.len() != 3 {
                return Err(UnicodeDataError);
            }
            let start = parse_hex(fields[1])?;
            match fields[0] {
                "F" => data.insert_fold(start, fields[2])?,
                "W" => insert_range(&mut data.whitespace, start, fields[2])?,
                "L" => insert_range(&mut data.letters_digits, start, fields[2])?,
                "M" => insert_range(&mut data.marks, start, fields[2])?,
                "K" => insert_range(&mut data.cjk, start, fields[2])?,
                _ => return Err(UnicodeDataError),
            }
        }
        Ok(data)
    }

    fn insert_fold(&mut self, start: u32, values: &str) -> Result<(), UnicodeDataError> {
        let original = char::from_u32(start).ok_or(UnicodeDataError)?;
        let folded = values
            .split_whitespace()
            .map(|value| char::from_u32(parse_hex(value)?).ok_or(UnicodeDataError))
            .collect::<Result<Vec<_>, _>>()?;
        if folded.is_empty() || self.folds.insert(original, folded).is_some() {
            return Err(UnicodeDataError);
        }
        Ok(())
    }

    /// Test pinned White_Space, independent of the Rust compiler's UCD version.
    pub fn is_whitespace(&self, character: char) -> bool {
        contains(&self.whitespace, character)
    }

    /// Test Letter or Decimal_Number.
    pub fn is_letter_or_digit(&self, character: char) -> bool {
        contains(&self.letters_digits, character)
    }

    /// Test a combining mark (General_Category Mn, Mc or Me).
    pub fn is_combining_mark(&self, character: char) -> bool {
        contains(&self.marks, character)
    }

    // @spec AC-006: Word characters include combining marks — .specs/features/002-word-replacements/spec.md#ac-006
    /// Test a word character for AC-006 boundaries: letter, decimal digit or
    /// combining mark, so a dependent vowel, virama or decomposed accent keeps
    /// its base letter's word intact.
    pub fn is_word_character(&self, character: char) -> bool {
        self.is_letter_or_digit(character) || self.is_combining_mark(character)
    }

    /// Test Han/kana/hangul and shared kana letters/marks for a side exemption.
    pub fn is_cjk(&self, character: char) -> bool {
        contains(&self.cjk, character)
    }

    /// Trim only Unicode White_Space at string edges, preserving inner bytes.
    pub fn trim_whitespace<'a>(&self, text: &'a str) -> &'a str {
        text.trim_matches(|character| self.is_whitespace(character))
    }

    // @spec AC-002: Full folded whitespace key — .specs/features/002-word-replacements/spec.md#ac-002
    /// Derive the unique recognized key without NFC, NFKC or accent removal.
    pub fn normalized_key(&self, text: &str) -> String {
        self.fold_with_offsets(self.trim_whitespace(text))
            .into_iter()
            .map(|token| token.character)
            .collect()
    }

    // @spec AC-006: Folding with source offsets — .specs/features/002-word-replacements/spec.md#ac-006
    /// Fold C+F non-Turkic mappings and collapse nonempty whitespace runs.
    /// Expansion flags prohibit matching only part of one original scalar.
    pub fn fold_with_offsets(&self, text: &str) -> Vec<FoldedToken> {
        let mut result = Vec::with_capacity(text.chars().count());
        let mut characters = text.char_indices().peekable();
        while let Some((start, character)) = characters.next() {
            let mut end = start + character.len_utf8();
            if self.is_whitespace(character) {
                // A whole whitespace run is one match token but consumes its original bytes.
                while let Some(&(offset, next)) = characters.peek() {
                    if !self.is_whitespace(next) {
                        break;
                    }
                    end = offset + next.len_utf8();
                    characters.next();
                }
                append_tokens(&mut result, &[' '], start, end);
            } else if let Some(folded) = self.folds.get(&character) {
                append_tokens(&mut result, folded, start, end);
            } else {
                append_tokens(&mut result, &[character], start, end);
            }
        }
        result
    }
}

fn parse_hex(value: &str) -> Result<u32, UnicodeDataError> {
    u32::from_str_radix(value, 16).map_err(|_| UnicodeDataError)
}

fn insert_range(
    ranges: &mut Vec<(u32, u32)>,
    start: u32,
    end: &str,
) -> Result<(), UnicodeDataError> {
    let end = parse_hex(end)?;
    if start > end || ranges.last().is_some_and(|previous| previous.1 >= start) {
        return Err(UnicodeDataError);
    }
    ranges.push((start, end));
    Ok(())
}

fn contains(ranges: &[(u32, u32)], character: char) -> bool {
    let value = character as u32;
    let index = ranges.partition_point(|(_, end)| *end < value);
    ranges.get(index).is_some_and(|(start, _)| *start <= value)
}

fn append_tokens(output: &mut Vec<FoldedToken>, values: &[char], start: usize, end: usize) {
    output.extend(
        values
            .iter()
            .enumerate()
            .map(|(index, character)| FoldedToken {
                character: *character,
                source_start: start,
                source_end: end,
                is_source_start: index == 0,
                is_source_end: index + 1 == values.len(),
            }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_embedded_data_and_license_against_pinned_provenance(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use sha2::{Digest, Sha256};
        #[derive(serde::Deserialize)]
        struct Identity {
            sha256: String,
        }
        #[derive(serde::Deserialize)]
        struct Provenance {
            version: String,
            generated_sha256: String,
            license: Identity,
        }
        let provenance: Provenance = serde_json::from_str(include_str!("unicode-sources.json"))?;
        assert_eq!(provenance.version, UNICODE_VERSION);
        assert_eq!(
            format!("{:x}", Sha256::digest(include_bytes!("unicode-data.txt"))),
            provenance.generated_sha256
        );
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(include_bytes!("unicode-license.txt"))
            ),
            provenance.license.sha256
        );
        Ok(())
    }

    #[test]
    fn fold_default_unicode_without_turkic_or_normalization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = UnicodeData::load()?;
        for (input, expected) in [
            ("Straße", "strasse"),
            ("ﬃ", "ffi"),
            ("Σςσ", "σσσ"),
            ("İIı", "i\u{307}iı"),
            ("CAFÉ", "café"),
            ("Cafe\u{301}", "cafe\u{301}"),
            ("𐐀", "𐐨"),
            ("  Cloud\u{A0}\u{2003}CODE  ", "cloud code"),
        ] {
            assert_eq!(data.normalized_key(input), expected, "{input}");
        }
        assert_ne!(data.normalized_key("é"), data.normalized_key("e\u{301}"));
        assert_ne!(data.normalized_key("café"), data.normalized_key("cafe"));
        Ok(())
    }

    #[test]
    fn retain_complete_expansion_and_whitespace_source_boundaries(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = UnicodeData::load()?;
        let text = "𐐀ß\u{A0} \tİ";
        let folded = data.fold_with_offsets(text);
        assert_eq!(
            folded
                .iter()
                .map(|token| token.character)
                .collect::<String>(),
            "𐐨ss i\u{307}"
        );
        assert_eq!((folded[0].source_start, folded[0].source_end), (0, 4));
        assert_eq!((folded[1].source_start, folded[1].source_end), (4, 6));
        assert!(folded[1].is_source_start && !folded[1].is_source_end);
        assert!(!folded[2].is_source_start && folded[2].is_source_end);
        assert_eq!(
            &text[folded[3].source_start..folded[3].source_end],
            "\u{A0} \t"
        );
        assert!(folded[3].is_source_start && folded[3].is_source_end);
        assert!(!folded[4].is_source_end && !folded[5].is_source_start);
        Ok(())
    }

    #[test]
    fn classify_pinned_supplementary_letters_decimal_digits_and_cjk(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = UnicodeData::load()?;
        for character in ['𐐀', '𠀀', '٣', '𝟚', '\u{1E4F0}', '\u{323B0}'] {
            assert!(data.is_letter_or_digit(character), "{character}");
        }
        for character in ['_', '²', '\u{301}', '。'] {
            assert!(!data.is_letter_or_digit(character), "{character}");
        }
        for character in ['𠀀', '漢', 'か', 'ｶ', '한', 'ᄀ', 'ー', 'ｰ', '\u{3099}'] {
            assert!(data.is_cjk(character), "{character}");
        }
        for character in ['a', '。', '、', '·', '2'] {
            assert!(!data.is_cjk(character), "{character}");
        }
        assert!(data.is_whitespace('\u{2007}'));
        assert!(!data.is_whitespace('\u{200B}'));
        Ok(())
    }

    #[test]
    fn classify_pinned_combining_marks_as_word_characters() -> Result<(), Box<dyn std::error::Error>>
    {
        let data = UnicodeData::load()?;
        // Mn (acute, virama), Mc (dependent vowels), Me (enclosing circle).
        for character in ['\u{301}', '\u{94D}', '\u{93E}', '\u{940}', '\u{20DD}'] {
            assert!(data.is_combining_mark(character), "{character}");
            assert!(data.is_word_character(character), "{character}");
        }
        for character in ['a', '_', '²', '\u{200D}', '。'] {
            assert!(!data.is_combining_mark(character), "{character}");
        }
        assert!(data.is_word_character('a') && data.is_word_character('٣'));
        assert!(!data.is_word_character('_') && !data.is_word_character('²'));
        Ok(())
    }

    #[test]
    fn reject_malformed_tables_without_panicking() {
        for source in [
            "F;DF;",
            "F;ZZ;73",
            "F;110000;73",
            "W;20;1F",
            "W;20;20\nW;20;21",
            "M;301;300",
            "Q;20;20",
        ] {
            assert!(UnicodeData::parse(source).is_err());
        }
    }
}
