//! Contains helper macros for working with the i18n API

/// Wrapper macro for [`crate::Locale::translate`] and [`crate::Locale::translate_with_arguments`].
///
/// # Examples
/// ```no_run
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
        // Pre-allocate HashMap with known capacity for better performance
        let mut args = std::collections::HashMap::with_capacity(t!(@count $($key)+));
        $(
            args.insert(
                std::borrow::Cow::Borrowed($key),
                $crate::FluentValue::from($value)
            );
        )+
        $crate::LOCALE.translate_with_arguments($id, &args)
    }};

    // Helper macro to count arguments at compile time
    (@count) => { 0 };
    (@count $head:tt $($tail:tt)*) => { 1 + t!(@count $($tail)*) };
}
