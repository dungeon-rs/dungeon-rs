mod extraction;
mod usage;

use crate::translations::usage::{TranslationUsage, TranslationUsageVisitor};
use anyhow::Context;
use cargo_metadata::{Metadata, camino::Utf8PathBuf};
use cli_colors::Colorizer;
use fluent::FluentResource;
use std::collections::HashMap;
use std::fs;
use syn::visit::Visit;
use walkdir::WalkDir;

pub fn execute(_colorizer: Colorizer, metadata: Metadata) -> anyhow::Result<()> {
    let workspace_root = &metadata.workspace_root;
    let _usage = find_translation_usages(workspace_root)?;
    let _translations = load_translations(workspace_root)?;

    Ok(())
}

/// Find all translation usages (with arguments) in t! macro calls throughout the workspace
fn find_translation_usages(workspace_root: &Utf8PathBuf) -> anyhow::Result<Vec<TranslationUsage>> {
    let mut usages = Vec::new();

    // Walk through all Rust source files in the workspace
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("Failed to read file: {}", entry.path().display()))?;

        // Parse the Rust file and extract t! macro usages
        if let Ok(file) = syn::parse_file(&content) {
            let mut visitor = TranslationUsageVisitor::new(entry.path().display().to_string());
            visitor.visit_file(&file);
            usages.extend(visitor.usages);
        }
    }

    Ok(usages)
}

/// Loads all translations from .ftl files in the locales directory
fn load_translations(
    workspace_root: &Utf8PathBuf,
) -> anyhow::Result<HashMap<String, HashMap<String, Vec<String>>>> {
    let mut translations = HashMap::new();
    let locales_dir = workspace_root.join("locales");

    for entry in WalkDir::new(&locales_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
    {
        let path = entry.path();

        // Extract language from the path (assuming structure: locales/lang/file.ftl)
        let language = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read fluent file: {}", path.display()))?;
        let keys = load_translation(content)
            .with_context(|| format!("Failed to load translation: {}", path.display()))?;
        translations
            .entry(language.to_string())
            .or_insert_with(HashMap::new)
            .extend(keys);
    }

    Ok(translations)
}

/// Extracts all translation keys from a Fluent file
fn load_translation(content: String) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let resource = FluentResource::try_new(content)
        .map_err(|(_, errors)| anyhow::anyhow!("Failed to parse Fluent resource: {:?}", errors))?;

    extraction::extract_from_resource(resource)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_simple_messages() {
        let source = String::from("hello = Hello, world!");

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(translations["hello"], Vec::<String>::new());
    }

    #[test]
    fn can_parse_message_with_argument() {
        let source = String::from("hello = Hello, { $name }!");

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(translations["hello"], vec![String::from("name")]);
    }

    #[test]
    fn can_parse_message_with_arguments() {
        let source = String::from("hello = Hello, {$name} { $lastname }!");

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(
            translations["hello"],
            vec![String::from("name"), String::from("lastname")]
        );
    }

    #[test]
    fn can_parse_message_with_multiline() {
        let source = String::from(
            r#"single = Text can be written in a single line.

multi = Text can also span multiple lines
    as long as each new line is indented
    by at least one space.

block =
    Sometimes it's more readable to format
    multiline text as a "block", which means
    starting it on a new line. All lines must
    be indented by at least one space."#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 3);
        assert_eq!(translations["single"], Vec::<String>::new());
        assert_eq!(translations["multi"], Vec::<String>::new());
        assert_eq!(translations["block"], Vec::<String>::new());
    }

    #[test]
    fn can_parse_message_with_comments() {
        let source = String::from("# this is a comment");

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 0);
    }

    #[test]
    fn can_parse_complex_message() {
        let source = String::from(
            r#"# $duration (Number) - The duration in seconds.
time-elapsed = Time elapsed: { NUMBER($duration, maximumFractionDigits: 0) }s."#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(translations["time-elapsed"], vec![String::from("duration")]);
    }

    #[test]
    fn can_parse_inline_references() {
        let source = String::from(
            r#"-brand-name = Firefox
installing = Installing { -brand-name }."#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(translations["installing"], Vec::<String>::new());
    }

    #[test]
    fn can_parse_selectors() {
        let source = String::from(
            r#"emails =
    { $unreadEmails ->
        [one] You have one unread email.
       *[other] You have { $unreadEmails } unread emails.
    }"#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 1);
        assert_eq!(translations["emails"], vec![String::from("unreadEmails")]);
    }

    #[test]
    fn can_parse_attributes() {
        let source = String::from(
            r#"login-input = Predefined value
    .placeholder = email@example.com
    .aria-label = Login input value
    .title = Type your login email"#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 4);
    }

    #[test]
    fn can_parse_attributes_with_variables() {
        let source = String::from(
            r#"login-input = Predefined value
    .placeholder = {$email}
    .aria-label = Login input value
    .title = Type your login email"#,
        );

        let translations = load_translation(source).expect("Failed to parse simple message");
        assert_eq!(translations.len(), 4);
        assert_eq!(
            translations["login-input.placeholder"],
            vec![String::from("email")]
        );
    }
}
