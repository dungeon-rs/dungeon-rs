//! This module contains the command to validate that every WGSL Shader a crate bundles parses
//! and validates, so a mistake in one shows without a GPU rather than when Bevy first compiles it.
use crate::violation::Violation;
use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use std::fs;
use std::path::{Path, PathBuf};

const RULE: &str = "WGSL Shader";

/// Where the crates live, relative to the workspace root.
const CRATES: &str = "crates";

/// The name of a directory whose WGSL files are Shaders.
const SHADERS: &str = "shaders";

/// Every `*.wgsl` file in a `shaders` directory anywhere under `crates/` parses as WGSL and
/// passes naga's validation, the one wgpu runs before it compiles a Shader.
///
/// A workspace without a `crates` directory has nothing to check.
pub fn check(metadata: &Metadata) -> Result<Vec<Violation>> {
    let root = metadata.workspace_root.as_std_path();
    let mut shaders = Vec::new();
    let crates = root.join(CRATES);
    if crates.is_dir() {
        find(&crates, &mut shaders)?;
    }
    shaders.sort();

    let mut violations = Vec::new();
    for path in shaders {
        let source =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        if let Some(problem) = problem(&source) {
            let name = path.strip_prefix(root).unwrap_or(&path);
            violations.push(Violation::new(
                name.to_string_lossy().replace('\\', "/"),
                RULE,
                problem,
            ));
        }
    }
    Ok(violations)
}

/// Collects every WGSL file in a `shaders` directory under `directory`.
fn find(directory: &Path, shaders: &mut Vec<PathBuf>) -> Result<()> {
    let in_shaders = directory.file_name().is_some_and(|name| name == SHADERS);
    for entry in
        fs::read_dir(directory).with_context(|| format!("listing {}", directory.display()))?
    {
        let path = entry
            .with_context(|| format!("listing {}", directory.display()))?
            .path();
        if path.is_dir() {
            find(&path, shaders)?;
        } else if in_shaders
            && path
                .extension()
                .is_some_and(|extension| extension == "wgsl")
        {
            shaders.push(path);
        }
    }
    Ok(())
}

/// What is wrong with the Shader `source`, if anything: where it fails to parse or what fails
/// to validate.
fn problem(source: &str) -> Option<String> {
    let module = match naga::front::wgsl::parse_str(source) {
        Ok(module) => module,
        Err(error) => {
            return Some(match error.location(source) {
                Some(at) => format!(
                    "does not parse at line {}: {}",
                    at.line_number,
                    error.message()
                ),
                None => format!("does not parse: {}", error.message()),
            });
        }
    };
    let error = Validator::new(ValidationFlags::all(), Capabilities::default())
        .validate(&module)
        .err()?;
    Some(match error.location(source) {
        Some(at) => format!("does not validate at line {}: {error}", at.line_number),
        None => format!("does not validate: {error}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Crate, workspace};

    /// A fragment Shader that parses and validates.
    const VALID: &str =
        "@fragment\nfn fragment() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0);\n}\n";

    /// Writes `source` as `crates/drs-render-engine/<directory>/<name>` in the fixture workspace
    /// at `root`.
    fn write(root: &Path, directory: &str, name: &str, source: &str) {
        let directory = root.join("crates/drs-render-engine").join(directory);
        fs::create_dir_all(&directory).expect("create the directory");
        fs::write(directory.join(name), source).expect("write the Shader");
    }

    #[test]
    fn a_valid_shader_passes() {
        let (dir, metadata) = workspace(&[Crate::new("drs-render-engine")]);
        write(dir.path(), "src/shaders", "valid.wgsl", VALID);

        assert!(check(&metadata).expect("checks").is_empty());
    }

    #[test]
    fn a_shader_that_does_not_parse_is_named_with_its_line() {
        let (dir, metadata) = workspace(&[Crate::new("drs-render-engine")]);
        write(
            dir.path(),
            "src/shaders",
            "broken.wgsl",
            "@fragment\nfn fragment() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0)\n",
        );

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].package,
            "crates/drs-render-engine/src/shaders/broken.wgsl"
        );
        assert_eq!(violations[0].rule, "WGSL Shader");
        assert!(
            violations[0].detail.starts_with("does not parse at line"),
            "{}",
            violations[0].detail
        );
    }

    #[test]
    fn a_shader_that_does_not_validate_is_named() {
        let (dir, metadata) = workspace(&[Crate::new("drs-render-engine")]);
        write(
            dir.path(),
            "src/shaders",
            "mistyped.wgsl",
            "@fragment\nfn fragment() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0);\n}\n\nfn half(x: f32) -> f32 {\n    return x;\n}\n\nfn wrong() -> i32 {\n    return half(2.0);\n}\n",
        );

        let violations = check(&metadata).expect("checks");

        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].package,
            "crates/drs-render-engine/src/shaders/mistyped.wgsl"
        );
        assert!(
            violations[0].detail.starts_with("does not validate"),
            "{}",
            violations[0].detail
        );
    }

    #[test]
    fn only_wgsl_files_in_a_shaders_directory_are_checked() {
        let (dir, metadata) = workspace(&[Crate::new("drs-render-engine")]);
        write(dir.path(), "src", "elsewhere.wgsl", "not a shader");
        write(dir.path(), "src/shaders", "notes.txt", "not a shader");
        write(dir.path(), "assets/shaders", "valid.wgsl", VALID);

        assert!(check(&metadata).expect("checks").is_empty());
    }

    #[test]
    fn a_workspace_without_shaders_passes() {
        let (_dir, metadata) = workspace(&[Crate::new("drs-render-engine")]);

        assert!(check(&metadata).expect("checks").is_empty());
    }
}
