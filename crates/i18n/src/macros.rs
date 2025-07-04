//! Contains helper macros for working with the i18n API

/// Wrapper macro for [`crate::Locale::translate`] and [`crate::Locale::translate_with_arguments`].
///
/// # Examples
/// ```rust
/// # use i18n::t;
/// # pub fn main() {
/// // without parameters:
/// let _ = t!("hello");
/// // with parameters:
/// let _ = t!("hello", "name" => "your-name");
/// # }
/// ```
#[macro_export]
macro_rules! t {
    // Simple case: t!("message-id")
    ($id:expr) => {
        $crate::LOCALE.translate($id)
    };

    // With arguments: t!("message-id", "key" => value, "key2" => value2)
    ($id:expr, $($key:expr => $value:expr),+ $(,)?) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(
                std::borrow::Cow::Borrowed($key),
                fluent_templates::fluent_bundle::FluentValue::from($value)
            );
        )+
        $crate::LOCALE.translate_with_arguments($id, &args)
    }};
}
