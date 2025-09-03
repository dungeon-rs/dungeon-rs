mod extraction;
mod usage;

use crate::translations::usage::{TranslationUsage, TranslationUsageVisitor};
use anyhow::Context;
use cargo_metadata::{Metadata, camino::Utf8PathBuf};
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table, format::Justify, print_stdout};
use fluent::FluentResource;
use std::collections::{HashMap, HashSet};
use std::fs;
use syn::visit::Visit;
use walkdir::WalkDir;

pub fn execute(colorizer: Colorizer, metadata: Metadata) -> anyhow::Result<()> {
    let workspace_root = &metadata.workspace_root;

    // Find all translation usages in source code
    let translation_usages = find_translation_usages(workspace_root)?;
    let used_keys: HashSet<String> = translation_usages.iter().map(|u| u.key.clone()).collect();

    // Load all available translations from .ftl files
    let available_translations = load_translations(workspace_root)?;

    // Find validation issues
    let missing_keys = find_missing_translation_keys(&used_keys, &available_translations);
    let unused_keys = find_unused_translation_keys(&used_keys, &available_translations);
    let argument_errors =
        validate_translation_arguments(&translation_usages, &available_translations)?;

    // Display results
    if missing_keys.is_empty() && unused_keys.is_empty() && argument_errors.is_empty() {
        println!(
            "{}",
            colorizer.green(
                "All translation keys are properly defined, used, and have correct arguments 🎉"
            )
        );

        return Ok(());
    }

    if !missing_keys.is_empty() {
        display_missing_keys(&missing_keys, &colorizer)?;
    }

    if !unused_keys.is_empty() {
        if !missing_keys.is_empty() {
            println!();
        }

        display_unused_keys(&unused_keys, &colorizer)?;
    }

    if !argument_errors.is_empty() {
        if !missing_keys.is_empty() || !unused_keys.is_empty() {
            println!();
        }

        display_argument_errors(&argument_errors, &colorizer)?;
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct MissingTranslationKey {
    key: String,
    language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnusedTranslationKey {
    key: String,
    language: String,
}

#[derive(Debug)]
enum ArgumentValidationError {
    MissingRequiredArgument {
        key: String,
        missing_arg: String,
        file: String,
    },
    UnexpectedArgument {
        key: String,
        unexpected_arg: String,
        file: String,
    },
}

/// Find translation keys that are used in code but missing from translation files
fn find_missing_translation_keys(
    used_keys: &HashSet<String>,
    available_translations: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Vec<MissingTranslationKey> {
    let mut missing = Vec::new();

    for key in used_keys {
        // Check if key is missing in all languages (not defined anywhere)
        let is_missing_everywhere = !available_translations
            .values()
            .any(|translations| translations.contains_key(key));

        if is_missing_everywhere {
            missing.push(MissingTranslationKey {
                key: key.clone(),
                language: "ALL".to_string(),
            });
        } else {
            // Check individual languages if the key exists somewhere
            for (language, translations) in available_translations {
                if !translations.contains_key(key) {
                    missing.push(MissingTranslationKey {
                        key: key.clone(),
                        language: language.clone(),
                    });
                }
            }
        }
    }

    // Sort by key, then by language
    missing.sort_by(|a, b| a.key.cmp(&b.key).then(a.language.cmp(&b.language)));
    missing.dedup();

    missing
}

/// Find translation keys that are defined in files but never used in code
fn find_unused_translation_keys(
    used_keys: &HashSet<String>,
    available_translations: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Vec<UnusedTranslationKey> {
    let mut unused = Vec::new();

    for (language, translations) in available_translations {
        for key in translations.keys() {
            if !used_keys.contains(key) {
                unused.push(UnusedTranslationKey {
                    key: key.clone(),
                    language: language.clone(),
                });
            }
        }
    }

    // Sort by key, then by language
    unused.sort_by(|a, b| a.key.cmp(&b.key).then(a.language.cmp(&b.language)));

    unused
}

/// Validate that translation usages have correct arguments
fn validate_translation_arguments(
    usages: &[TranslationUsage],
    available_translations: &HashMap<String, HashMap<String, Vec<String>>>,
) -> anyhow::Result<Vec<ArgumentValidationError>> {
    let mut errors = Vec::new();

    for usage in usages {
        // Find the translation definition (use any language that has it)
        let empty_args = Vec::new();
        let translation_args = available_translations
            .values()
            .find_map(|translations| translations.get(&usage.key))
            .unwrap_or(&empty_args);

        if !translation_args.is_empty() || !usage.arguments.is_empty() {
            let provided_args: HashSet<_> = usage.arguments.iter().collect();
            let required_args: HashSet<_> = translation_args.iter().collect();

            // Check for missing required arguments
            for required_arg in &required_args {
                if !provided_args.contains(required_arg) {
                    errors.push(ArgumentValidationError::MissingRequiredArgument {
                        key: usage.key.clone(),
                        missing_arg: required_arg.to_string(),
                        file: usage.file_path.clone(),
                    });
                }
            }

            // Check for unexpected arguments
            for provided_arg in &provided_args {
                if !required_args.contains(provided_arg) {
                    errors.push(ArgumentValidationError::UnexpectedArgument {
                        key: usage.key.clone(),
                        unexpected_arg: provided_arg.to_string(),
                        file: usage.file_path.clone(),
                    });
                }
            }
        }
    }

    Ok(errors)
}

/// Display missing translation keys in a formatted table
fn display_missing_keys(
    missing: &[MissingTranslationKey],
    colorizer: &Colorizer,
) -> anyhow::Result<()> {
    println!("{}", colorizer.red("✗ Missing translation keys found:"));
    println!();

    let table = missing
        .iter()
        .map(|mt| {
            vec![
                mt.key.clone().cell().justify(Justify::Left),
                mt.language.clone().cell().justify(Justify::Center),
            ]
        })
        .table()
        .title(vec![
            "Translation Key".cell().bold(true),
            "Missing in Language".cell().bold(true),
        ])
        .bold(true);

    print_stdout(table)?;

    println!();
    println!(
        "{}",
        colorizer.yellow(format!(
            "Found {} missing translation key(s)",
            missing.len()
        ))
    );

    Ok(())
}

/// Display unused translation keys in a formatted table
fn display_unused_keys(
    unused: &[UnusedTranslationKey],
    colorizer: &Colorizer,
) -> anyhow::Result<()> {
    println!("{}", colorizer.yellow("⚠  Unused translation keys found:"));
    println!();

    let table = unused
        .iter()
        .map(|ut| {
            vec![
                ut.key.clone().cell().justify(Justify::Left),
                ut.language.clone().cell().justify(Justify::Center),
            ]
        })
        .table()
        .title(vec![
            "Translation Key".cell().bold(true),
            "Defined in Language".cell().bold(true),
        ])
        .bold(true);

    print_stdout(table)?;

    println!();
    println!(
        "{}",
        colorizer.yellow(format!("Found {} unused translation key(s)", unused.len()))
    );

    Ok(())
}

/// Display argument validation errors in a formatted table
fn display_argument_errors(
    errors: &[ArgumentValidationError],
    colorizer: &Colorizer,
) -> anyhow::Result<()> {
    println!("{}", colorizer.red("✗ Translation argument errors found:"));
    println!();

    let table = errors
        .iter()
        .map(|error| match error {
            ArgumentValidationError::MissingRequiredArgument {
                key,
                missing_arg,
                file,
            } => {
                vec![
                    key.cell().justify(Justify::Left),
                    format!("Missing: {missing_arg}")
                        .cell()
                        .justify(Justify::Left),
                    file.cell().justify(Justify::Left),
                ]
            }
            ArgumentValidationError::UnexpectedArgument {
                key,
                unexpected_arg,
                file,
            } => {
                vec![
                    key.cell().justify(Justify::Left),
                    format!("Unexpected: '{}'", colorizer.blue(unexpected_arg))
                        .cell()
                        .justify(Justify::Left),
                    file.cell().justify(Justify::Left),
                ]
            }
        })
        .table()
        .title(vec![
            "Translation Key".cell().bold(true),
            "Argument Issue".cell().bold(true),
            "File".cell().bold(true),
        ])
        .bold(true);

    print_stdout(table)?;

    println!();
    println!(
        "{}",
        colorizer.yellow(format!("Found {} argument error(s)", errors.len()))
    );

    Ok(())
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
