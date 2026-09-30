//! Builds small throwaway workspaces so each check can be run against a violation of its rule.
use cargo_metadata::{Metadata, MetadataCommand};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// One crate in a fixture workspace.
pub struct Crate {
    name: String,
    /// Workspace crates (by name) or external crates this crate depends on.
    dependencies: Vec<String>,
    features: Vec<(String, Vec<String>)>,
    /// `None` leaves the README out.
    readme: Option<String>,
}

impl Crate {
    /// A crate with the required features, the given README text, and no dependencies.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dependencies: Vec::new(),
            features: vec![("default".into(), vec![]), ("dev".into(), vec![])],
            readme: Some("# crate\n\nFeatures: `default`, `dev`.\n".into()),
        }
    }

    /// The crate's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a dependency on a workspace crate; `dev` is propagated to it.
    pub fn depends_on(mut self, name: &str) -> Self {
        self.dependencies.push(name.to_string());
        self.features
            .iter_mut()
            .find(|(feature, _)| feature == "dev")
            .expect("new crates declare dev")
            .1
            .push(format!("{name}/dev"));
        self
    }

    /// Adds a dependency on a workspace crate without propagating `dev`.
    pub fn depends_on_without_dev(mut self, name: &str) -> Self {
        self.dependencies.push(name.to_string());
        self
    }

    /// Adds a dependency on an external crate.
    pub fn depends_on_external(mut self, name: &str) -> Self {
        self.dependencies.push(format!("external:{name}"));
        self
    }

    /// Replaces the declared features.
    pub fn with_features(mut self, features: &[(&str, &[&str])]) -> Self {
        self.features = features
            .iter()
            .map(|(name, enables)| {
                (
                    (*name).to_string(),
                    enables.iter().map(|e| (*e).to_string()).collect(),
                )
            })
            .collect();
        self
    }

    pub fn with_readme(mut self, readme: &str) -> Self {
        self.readme = Some(readme.to_string());
        self
    }

    pub fn without_readme(mut self) -> Self {
        self.readme = None;
        self
    }
}

/// Writes the crates to a temporary workspace and returns its `cargo metadata`.
///
/// The returned [`TempDir`] must be kept alive while the metadata is read from disk.
pub fn workspace(crates: &[Crate]) -> (TempDir, Metadata) {
    let dir = TempDir::new().expect("create temporary workspace");
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"crates/*\"]\n",
    )
    .expect("write workspace manifest");

    for krate in crates {
        write_crate(dir.path(), krate);
    }

    let metadata = MetadataCommand::new()
        .manifest_path(dir.path().join("Cargo.toml"))
        .no_deps()
        .exec()
        .expect("read fixture metadata");

    (dir, metadata)
}

fn write_crate(root: &Path, krate: &Crate) {
    let dir = root.join("crates").join(&krate.name);
    fs::create_dir_all(dir.join("src")).expect("create crate directory");
    fs::write(dir.join("src/lib.rs"), "").expect("write crate root");

    let mut manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n",
        krate.name
    );
    for dependency in &krate.dependencies {
        match dependency.strip_prefix("external:") {
            Some(external) => manifest.push_str(&format!("{external} = \"1\"\n")),
            None => manifest.push_str(&format!(
                "{dependency} = {{ path = \"../{dependency}\" }}\n"
            )),
        }
    }
    manifest.push_str("\n[features]\n");
    for (feature, enables) in &krate.features {
        let enables: Vec<String> = enables.iter().map(|e| format!("\"{e}\"")).collect();
        manifest.push_str(&format!("{feature} = [{}]\n", enables.join(", ")));
    }
    fs::write(dir.join("Cargo.toml"), manifest).expect("write crate manifest");

    if let Some(readme) = &krate.readme {
        fs::write(dir.join("README.md"), readme).expect("write crate README");
    }
}
