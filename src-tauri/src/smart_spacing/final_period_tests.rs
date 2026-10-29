//! Final period inside a sentence (spec AC-017, decision `n30_trailing_period`): what follows
//! the selection, which periods are a sentence's, and the characters never touched.
use super::*;

#[test]
fn punctuation_after_drops_the_final_period() {
    for after in [
        ", ça va",
        ". Merci",
        "; puis",
        ": voici",
        "! Oui",
        "? Non",
        ") fin",
        "] fin",
        "} fin",
        "” dit-il",
        "’ ok",
        "» dit-il",
        "› x",
        "\" dit-il",
        "  , ça va",
        "\u{a0}; puis",
    ] {
        assert_eq!(
            drop_final_period("tout le monde.", after),
            "tout le monde",
            "{after:?}"
        );
    }
}

#[test]
fn lower_case_after_drops_the_final_period() {
    for after in ["demain", " demain", "\tdemain", "écrit", "über", "и дальше"] {
        assert_eq!(drop_final_period("à tous.", after), "à tous", "{after:?}");
    }
}

#[test]
fn a_new_sentence_or_line_keeps_the_period() {
    for after in [
        "Demain",
        " Demain",
        "\nDemain",
        "\ndemain",
        "\r\nsuite",
        "  \nsuite",
        "3 jours",
        "東京",
        "\"Bonjour",
        "(voir",
        "- suite",
    ] {
        assert_eq!(drop_final_period("à tous.", after), "à tous.", "{after:?}");
    }
}

#[test]
fn nothing_after_keeps_the_period() {
    for after in ["", " ", "   ", "\u{a0}"] {
        assert_eq!(drop_final_period("à tous.", after), "à tous.", "{after:?}");
    }
}

#[test]
fn only_a_single_full_stop_is_dropped() {
    for body in [
        "à tous…",
        "à tous...",
        "à tous..",
        "à tous?",
        "à tous!",
        "à tous?.",
        "à tous).",
        "à tous\".",
        "你好。",
        "à tous．",
        ".",
        "à tous",
    ] {
        assert_eq!(drop_final_period(body, "demain"), body, "{body:?}");
    }
}

#[test]
fn abbreviations_keep_their_period() {
    for body in [
        "pommes, poires, etc.",
        "voir cf.",
        "Paris vs.",
        "M.",
        "chez M.",
        "J.",
        "aux U.S.",
        "e.g.",
        "la version 3.5.",
        "Mme.",
        "chez Mme.",
        "(etc.",
        "le Dr.",
    ] {
        assert_eq!(drop_final_period(body, ", ça va"), body, "{body:?}");
    }
}

#[test]
fn ordinary_words_and_numbers_lose_their_period() {
    for (body, expected) in [
        ("tout le monde.", "tout le monde"),
        ("chez Mme Dupont.", "chez Mme Dupont"),
        ("en 2026.", "en 2026"),
        ("USA.", "USA"),
        ("Ok.", "Ok"),
    ] {
        assert_eq!(drop_final_period(body, ", ça va"), expected, "{body:?}");
    }
}

#[test]
fn only_the_final_period_changes() {
    let body = "Oui. Bien sûr, à tous.";
    let dropped = drop_final_period(body, "demain");
    assert_eq!(format!("{dropped}."), body);
}
