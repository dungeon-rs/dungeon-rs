//! Text as compared ignoring letter case and Unicode normalisation.

use unicode_normalization::UnicodeNormalization;

/// `text` Unicode-normalised (NFC) and then fully case-folded, so that `Straße` and `STRASSE`
/// compare equal where lower-casing would not.
pub(crate) fn fold(text: &str) -> String {
    if text.is_ascii() {
        return text.to_ascii_lowercase();
    }
    caseless::default_case_fold_str(&text.nfc().collect::<String>())
}
