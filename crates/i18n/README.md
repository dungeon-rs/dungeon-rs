# DungeonRS i18n

A wrapper around `fluent-template` to make static access a bit more convenient.
It does this by providing a static `LOCALES` which wraps `ArcLoader` and tracks
the currently selected language. It also provides a `t!` macro which makes
using it even more intuitive.
