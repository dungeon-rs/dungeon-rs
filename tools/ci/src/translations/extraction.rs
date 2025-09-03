//! Contains the code for extracing translation keys and arguments from the Fluent files.

use anyhow::bail;
use fluent::FluentResource;
use fluent_syntax::ast::{Attribute, Entry, Expression, InlineExpression, Pattern, PatternElement};
use std::collections::HashMap;

/// Takes a FluentResource and proceeds to parse all translation keys and their corresponding arguments
/// from it.
pub fn extract_from_resource(
    resource: FluentResource,
) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let mut result = HashMap::new();

    for entry in resource.entries() {
        match entry {
            Entry::Message(message) => {
                let name = message.id.name.to_string();
                let arguments = extract_from_pattern(message.value.as_ref())?;

                result.insert(name.clone(), arguments.clone());
                for attribute in &message.attributes {
                    let attribute_name = attribute.id.name.to_string();
                    let arguments = extract_from_attribute(attribute)?;

                    result.insert(format!("{}.{}", name, attribute_name), arguments);
                }
            }
            Entry::Junk { content } => bail!("Invalid Fluent syntax: {}", content),
            _ => {}
        }
    }

    Ok(result)
}

/// A pattern is a value of a message, term or attribute.
/// https://docs.rs/fluent-syntax/0.12.0/fluent_syntax/ast/struct.Pattern.html
fn extract_from_pattern(pattern: Option<&Pattern<&str>>) -> anyhow::Result<Vec<String>> {
    let mut arguments = Vec::new();
    let Some(pattern) = pattern else {
        return Ok(arguments);
    };

    // For each element in the pattern we gather all variable names recursively.
    for element in pattern.elements.iter() {
        match element {
            // These are text literals, we don't need these.
            PatternElement::TextElement { .. } => {}
            PatternElement::Placeable { expression, .. } => {
                let names = extract_from_expression(expression)?;
                arguments.extend(names);
            }
        }
    }

    Ok(arguments)
}

/// Extract from an attribute.
/// https://projectfluent.org/fluent/guide/attributes.html
fn extract_from_attribute(attribute: &Attribute<&str>) -> anyhow::Result<Vec<String>> {
    extract_from_pattern(Some(&attribute.value))
}

/// Expressions can be either inline or select expressions.
/// https://docs.rs/fluent-syntax/0.12.0/fluent_syntax/ast/enum.Expression.html
fn extract_from_expression(expression: &Expression<&str>) -> anyhow::Result<Vec<String>> {
    let names = match expression {
        // key = { $var ->
        //   [key1] Value 1
        //   *[other] Value 2
        // }
        Expression::Select { selector, .. } => extract_from_inline_expression(selector)?,
        // hello-user = Hello ${ username }
        Expression::Inline(inline) => extract_from_inline_expression(inline)?,
    };

    Ok(names)
}

fn extract_from_inline_expression(
    expression: &InlineExpression<&str>,
) -> anyhow::Result<Vec<String>> {
    let names = match expression {
        // String literal; can be ignored.
        InlineExpression::StringLiteral { .. } => vec![],
        // Number literal; can be ignored.
        InlineExpression::NumberLiteral { .. } => vec![],
        // Reference to another message; can be ignored.
        InlineExpression::MessageReference { .. } => vec![],
        // Reference to inline variable; can be ignored.
        InlineExpression::TermReference { .. } => vec![],
        // Function call, arguments can be variables, so we'll need to recurse.
        InlineExpression::FunctionReference { arguments, .. } => {
            let mut names = vec![];
            // First we iterate over all positional arguments.
            for argument in arguments.positional.iter() {
                names.extend(extract_from_inline_expression(argument)?);
            }

            // Then we iterate over all named arguments.
            for argument in arguments.named.iter() {
                names.extend(extract_from_inline_expression(&argument.value)?);
            }

            names
        }
        // Variable, we need the name.
        InlineExpression::VariableReference { id } => vec![id.name.to_string()],
        // Recursive expression, we need to recurse.
        InlineExpression::Placeable { expression } => extract_from_expression(expression.as_ref())?,
    };

    Ok(names)
}
