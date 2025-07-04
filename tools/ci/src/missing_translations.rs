//! This module contains the command to validate that all translation keys in the app have been
//! translated in all available languages.

use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use cargo_metadata::camino::Utf8PathBuf;
use cli_colors::Colorizer;
use cli_table::{Cell, Style, Table, format::Justify, print_stdout};
use std::collections::{HashMap, HashSet};
use std::fs;
use syn::{Expr, ExprMacro, Lit, visit::Visit};
use walkdir::WalkDir;

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

    // Find all translation keys used in source code
    let used_keys = find_translation_keys(workspace_root)?;

    // Find all available translations from .ftl files
    let available_translations = load_fluent_translations(workspace_root)?;

    // Check for missing translations
    let missing_translations = find_missing_translations(&used_keys, &available_translations);

    // Check for unused translations
    let unused_translations = find_unused_translations(&used_keys, &available_translations);

    if missing_translations.is_empty() && unused_translations.is_empty() {
        println!(
            "{}",
            colorizer.green("All translation keys are properly defined and used 🎉")
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

    if missing_translations.is_empty() && unused_translations.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Find all translation keys used in t! macro calls throughout the workspace
fn find_translation_keys(workspace_root: &Utf8PathBuf) -> Result<HashSet<String>> {
    let mut keys = HashSet::new();

    // Walk through all Rust source files in the workspace
    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("Failed to read file: {}", entry.path().display()))?;

        // Parse the Rust file and extract t! macro keys
        if let Ok(file) = syn::parse_file(&content) {
            let mut visitor = TranslationKeyVisitor::new();
            visitor.visit_file(&file);
            keys.extend(visitor.keys);
        }
    }

    Ok(keys)
}

/// Visitor to extract translation keys from t! macro calls
struct TranslationKeyVisitor {
    keys: Vec<String>,
}

impl TranslationKeyVisitor {
    fn new() -> Self {
        Self { keys: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for TranslationKeyVisitor {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        // Check if this is a t! macro call
        if let Some(ident) = node.mac.path.get_ident() {
            if ident == "t" {
                // Extract the first token which should be the translation key
                if let Ok(Expr::Lit(expr_lit)) = syn::parse2::<Expr>(node.mac.tokens.clone()) {
                    if let Lit::Str(lit_str) = expr_lit.lit {
                        self.keys.push(lit_str.value());
                    }
                }
            }
        }

        // Continue visiting child nodes
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

/// Display missing translations in a formatted table
fn display_missing_translations(
    missing: &[MissingTranslation],
    colorizer: &Colorizer,
) -> Result<()> {
    println!("{}", colorizer.red("❌ Missing translations found:"));
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
    println!("{}", colorizer.yellow("❌ Unused translations found:"));
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
        colorizer.yellow(format!("ound {} unused translation(s)", unused.len()))
    );

    Ok(())
}
