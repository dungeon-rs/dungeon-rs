//! Contains the code for extracting translation usages from source code

use syn::ExprMacro;
use syn::visit::Visit;

#[derive(Debug, Clone)]
pub(super) struct TranslationUsage {
    key: String,
    arguments: Vec<String>,
    file_path: String,
}

/// Visitor to extract translation usages (keys + arguments) from t! macro calls
pub(super) struct TranslationUsageVisitor {
    pub(crate) usages: Vec<TranslationUsage>,
    current_file: String,
}

impl TranslationUsageVisitor {
    pub fn new(file_path: String) -> Self {
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
