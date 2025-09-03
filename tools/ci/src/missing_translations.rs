#![allow(unused)]
use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use cargo_metadata::camino::Utf8PathBuf;
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table, format::Justify, print_stdout};
use std::collections::{HashMap, HashSet};
use std::fs;
use syn::{ExprMacro, visit::Visit};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct TranslationUsage {
    key: String,
    arguments: Vec<String>,
    file_path: String,
}

#[derive(Debug, Clone)]
struct FluentMessage {
    required_args: HashSet<String>,
}

/// Visitor to extract translation usages (keys + arguments) from t! macro calls
struct TranslationUsageVisitor {
    usages: Vec<TranslationUsage>,
    current_file: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct MissingTranslation {
    key: String,
    language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnusedTranslation {
    key: String,
    language: String,
}

pub fn execute(colorizer: Colorizer, metadata: Metadata) -> Result<()> {
    let workspace_root = &metadata.workspace_root;

    // Find all translation usages in source code (with arguments)
    let translation_usages = find_translation_usages(workspace_root)?;
    let used_keys: HashSet<String> = translation_usages.iter().map(|u| u.key.clone()).collect();

    // Find all available translations from .ftl files
    let available_translations = load_fluent_translations(workspace_root)?;

    // Parse Fluent messages to get argument requirements
    let fluent_messages = parse_fluent_messages(workspace_root, &colorizer)?;

    // Check for missing translations
    let missing_translations = find_missing_translations(&used_keys, &available_translations);

    // Check for unused translations
    let unused_translations = find_unused_translations(&used_keys, &available_translations);

    // Check for argument validation errors
    let argument_errors = validate_translation_arguments(&translation_usages, &fluent_messages);

    if missing_translations.is_empty()
        && unused_translations.is_empty()
        && argument_errors.is_empty()
    {
        println!(
            "{}",
            colorizer.green(
                "All translation keys are properly defined, used, and have correct arguments 🎉"
            )
        );
        return Ok(());
    }

    // Display results in tables
    if !missing_translations.is_empty() {
        display_missing_translations(&missing_translations, &colorizer)?;
    }

    if !unused_translations.is_empty() {
        if !missing_translations.is_empty() {
            println!(); // Add spacing between tables
        }
        display_unused_translations(&unused_translations, &colorizer)?;
    }

    if !argument_errors.is_empty() {
        if !missing_translations.is_empty() || !unused_translations.is_empty() {
            println!(); // Add spacing between tables
        }
        display_argument_errors(&argument_errors, &colorizer)?;
    }

    Ok(())
}

/// Find all translation usages (with arguments) in t! macro calls throughout the workspace
fn find_translation_usages(workspace_root: &Utf8PathBuf) -> Result<Vec<TranslationUsage>> {
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

impl TranslationUsageVisitor {
    fn new(file_path: String) -> Self {
        Self {
            usages: Vec::new(),
            current_file: file_path,
        }
    }

    /// Extract both the translation key and arguments from macro tokens
    fn extract_translation_usage(
        &mut self,
        tokens: &proc_macro2::TokenStream,
    ) -> Option<TranslationUsage> {
        // Convert tokens to string and parse manually - this is more reliable for our specific macro format
        let token_string = tokens.to_string();

        // Split by commas to get individual arguments
        let parts: Vec<&str> = token_string.split(',').collect();

        if parts.is_empty() {
            return None;
        }

        // First part should be the key (remove quotes)
        let key = parts[0].trim().trim_matches('"').to_string();
        let mut arguments = Vec::new();

        // Parse remaining parts for "arg" => value patterns
        for part in parts.iter().skip(1) {
            let part = part.trim();
            if let Some(arrow_pos) = part.find("=>") {
                let arg_name = part[..arrow_pos].trim().trim_matches('"');
                if !arg_name.is_empty() {
                    arguments.push(arg_name.to_string());
                }
            }
        }

        Some(TranslationUsage {
            key,
            arguments,
            file_path: self.current_file.clone(),
        })
    }
}

impl<'ast> Visit<'ast> for TranslationUsageVisitor {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        if let Some(ident) = node.mac.path.get_ident() {
            if ident == "t" {
                if let Some(usage) = self.extract_translation_usage(&node.mac.tokens) {
                    self.usages.push(usage);
                }
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }
}

/// Load all translations from .ftl files in the locales directory
fn load_fluent_translations(
    workspace_root: &Utf8PathBuf,
) -> Result<HashMap<String, HashSet<String>>> {
    let mut translations = HashMap::new();
    let locales_dir = workspace_root.join("locales");

    if !locales_dir.exists() {
        return Ok(translations);
    }

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

        let keys = parse_fluent_keys(&content);
        translations
            .entry(language.to_string())
            .or_insert_with(HashSet::new)
            .extend(keys);
    }

    Ok(translations)
}

/// Parse Fluent file content to extract message keys
fn parse_fluent_keys(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                return None;
            }

            // Extract key from "key = value" format
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                if !key.is_empty() {
                    return Some(key.to_string());
                }
            }

            None
        })
        .collect()
}

/// Parse all Fluent files to extract message argument requirements
fn parse_fluent_messages(
    workspace_root: &Utf8PathBuf,
    colorizer: &Colorizer,
) -> Result<HashMap<String, FluentMessage>> {
    let mut fluent_messages = HashMap::new();
    let locales_dir = workspace_root.join("locales");

    if !locales_dir.exists() {
        return Ok(fluent_messages);
    }

    for entry in WalkDir::new(&locales_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
    {
        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("Failed to read fluent file: {}", entry.path().display()))?;

        // Parse each message in the file
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let message_content = line[eq_pos + 1..].trim();

                if !key.is_empty() {
                    let required_args = extract_fluent_variables(message_content, colorizer);
                    fluent_messages.insert(key.clone(), FluentMessage { required_args });
                }
            }
        }
    }

    Ok(fluent_messages)
}

/// Extract variable names from Fluent message content (e.g., {name}, {count})
fn extract_fluent_variables(content: &str, colorizer: &Colorizer) -> HashSet<String> {
    let mut variables = HashSet::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut var_name = String::new();

            while let Some(&next_ch) = chars.peek() {
                if next_ch == '$' || next_ch.is_whitespace() {
                    chars.next(); // consume '$' or whitespace
                } else if next_ch == '}' {
                    chars.next(); // consume '}'
                    break;
                } else if next_ch.is_alphanumeric() || next_ch == '_' || next_ch == '-' {
                    var_name.push(chars.next().unwrap());
                } else {
                    eprintln!(
                        "{} '{}'",
                        colorizer.red("Invalid character in variable name:"),
                        next_ch
                    );
                    // Invalid character in variable name, skip
                    break;
                }
            }

            if !var_name.is_empty() {
                variables.insert(var_name);
            }
        }
    }

    variables
}

/// Find missing translations by comparing used keys with available translations
fn find_missing_translations(
    used_keys: &HashSet<String>,
    available_translations: &HashMap<String, HashSet<String>>,
) -> Vec<MissingTranslation> {
    let mut missing = Vec::new();

    for key in used_keys {
        // Check if key is missing in all languages (not defined anywhere)
        let is_missing_everywhere = !available_translations
            .values()
            .any(|translations| translations.contains(key));

        if is_missing_everywhere {
            missing.push(MissingTranslation {
                key: key.clone(),
                language: "ALL".to_string(),
            });
        } else {
            // Only check individual languages if the key exists somewhere
            for (language, translations) in available_translations {
                if !translations.contains(key) {
                    missing.push(MissingTranslation {
                        key: key.clone(),
                        language: language.clone(),
                    });
                }
            }
        }
    }

    // Remove duplicates and sort
    missing.sort_by(|a, b| a.key.cmp(&b.key).then(a.language.cmp(&b.language)));
    missing.dedup();

    missing
}

/// Find translations that are defined but never used in code
fn find_unused_translations(
    used_keys: &HashSet<String>,
    available_translations: &HashMap<String, HashSet<String>>,
) -> Vec<UnusedTranslation> {
    let mut unused = Vec::new();

    for (language, translations) in available_translations {
        for key in translations {
            if !used_keys.contains(key) {
                unused.push(UnusedTranslation {
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

/// Validate that translation usages match their Fluent definitions
fn validate_translation_arguments(
    usages: &[TranslationUsage],
    fluent_messages: &HashMap<String, FluentMessage>,
) -> Vec<ArgumentValidationError> {
    let mut errors = Vec::new();

    for usage in usages {
        if let Some(message) = fluent_messages.get(&usage.key) {
            let provided_args: HashSet<_> = usage.arguments.iter().collect();
            let required_args: HashSet<_> = message.required_args.iter().collect();

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

    errors
}

/// Display missing translations in a formatted table
fn display_missing_translations(
    missing: &[MissingTranslation],
    colorizer: &Colorizer,
) -> Result<()> {
    println!("{}", colorizer.red("✗ Missing translations found:"));
    println!();

    let table = missing
        .iter()
        .map(|mt| {
            vec![
                mt.clone().key.cell().justify(Justify::Left),
                mt.clone().language.cell().justify(Justify::Center),
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
        colorizer.yellow(format!("Found {} missing translation(s)", missing.len()))
    );

    Ok(())
}

/// Display unused translations in a formatted table
fn display_unused_translations(unused: &[UnusedTranslation], colorizer: &Colorizer) -> Result<()> {
    println!("{}", colorizer.yellow("⚠ Unused translations found:"));
    println!();

    let table = unused
        .iter()
        .map(|ut| {
            vec![
                ut.clone().key.cell().justify(Justify::Left),
                ut.clone().language.cell().justify(Justify::Center),
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
        colorizer.yellow(format!("Found {} unused translation(s)", unused.len()))
    );

    Ok(())
}

/// Display argument validation errors in a formatted table
fn display_argument_errors(
    errors: &[ArgumentValidationError],
    colorizer: &Colorizer,
) -> Result<()> {
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
                    format!("Unexpected: {unexpected_arg}")
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
