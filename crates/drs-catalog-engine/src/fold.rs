//! Text as compared ignoring letter case and Unicode normalisation.

use unicode_normalization::UnicodeNormalization;

/// `text` Unicode-normalised (NFC), fully case-folded, and normalised again, so that `Straße` and
/// `STRASSE` compare equal where lower-casing would not, and so do two spellings whose letters
/// fold into different forms of the same text, as `Ϊ́` and `ΐ` do.
pub(crate) fn fold(text: &str) -> String {
    if text.is_ascii() {
        return text.to_ascii_lowercase();
    }
    caseless::default_case_fold_str(&text.nfc().collect::<String>())
        .nfc()
        .collect()
}
