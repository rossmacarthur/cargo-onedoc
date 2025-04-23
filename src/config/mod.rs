mod input;

use std::collections::HashMap;
use std::fs;
use std::io;

use anyhow::{anyhow, Context as _, Result};
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use cargo_metadata::{Metadata, Package, TargetKind};
use serde::Deserialize;
use serde::Serialize;

/// Configuration of which files to process.
#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    /// The badges configuration
    #[serde(default)]
    pub badges: Badges,

    /// A list of processes that each outputs a single Markdown file
    #[serde(default, rename = "doc")]
    pub docs: Vec<Doc>,

    /// Global link remapping config
    #[serde(default)]
    pub links: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

/// Badges to be added to the output file
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Badges {
    /// Whether to add a badge for the crates.io page
    #[serde(default = "default_true")]
    crates_io: bool,

    /// Whether to add a badge for the docs.rs page
    #[serde(default = "default_true")]
    docs_rs: bool,

    /// Whether to add a badge for the GitHub check
    #[serde(default)]
    github_workflow: Option<GitHubWorkflowBadge>,
}

impl Default for Badges {
    fn default() -> Self {
        Self {
            crates_io: true,
            docs_rs: true,
            github_workflow: Some(GitHubWorkflowBadge::default()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GitHubWorkflowBadge {
    label: String,
    name: String,
}

impl Default for GitHubWorkflowBadge {
    fn default() -> Self {
        Self {
            label: "build".to_owned(),
            name: "build".to_owned(),
        }
    }
}

// Custom deserialization for GitHubWorkflowBadge to make label default to name
impl<'de> Deserialize<'de> for GitHubWorkflowBadge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn default_build() -> String {
            "build".to_owned()
        }

        #[derive(Deserialize)]
        struct GitHubWorkflowBadgeHelper {
            label: Option<String>,
            #[serde(default = "default_build")]
            name: String,
        }

        let helper = GitHubWorkflowBadgeHelper::deserialize(deserializer)?;

        Ok(GitHubWorkflowBadge {
            label: helper.label.unwrap_or_else(|| helper.name.clone()),
            name: helper.name,
        })
    }
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Doc {
    /// A list of input file paths.
    ///
    /// Either absolute paths or relative to the Cargo workspace directory.
    #[serde(rename = "input", deserialize_with = "input::deserialize")]
    pub inputs: Vec<PathBuf>,

    /// The output file path.
    ///
    /// Either an absolute path or relative to the Cargo workspace directory.
    pub output: PathBuf,

    /// The template to render the processed Markdown
    pub template: Option<PathBuf>,
}

pub fn load(metadata: &Metadata, pkg: &Package) -> Result<Config> {
    let workspace_dir = &metadata.workspace_root;
    let path = workspace_dir.join("onedoc.toml");

    let mut config = {
        let ctx = || format!("failed to load config from `{}`", path);
        load_from_path(&path).with_context(ctx)?
    };

    // Make sure to specify at least one doc to process
    if config.docs.is_empty() {
        config.docs = vec![default_doc(pkg)?]
    }

    // Normalize all the paths
    for doc in &mut config.docs {
        for input in &mut doc.inputs {
            *input = workspace_dir.join(&input);
        }
        doc.output = workspace_dir.join(&doc.output);
        if let Some(p) = doc.template.as_mut() {
            *p = workspace_dir.join(&p);
        }
    }

    Ok(config)
}

fn load_from_path(path: &Path) -> Result<Config> {
    let config = match fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).context("failed to deserialize config")?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => Config::default(),
        Err(err) => return Err(err).context("failed to read config file")?,
    };
    Ok(config)
}

fn default_doc(pkg: &Package) -> Result<Doc> {
    let input = default_input_path(pkg)?;
    let output = default_output_path(pkg);
    let doc = Doc {
        inputs: vec![input],
        output,
        template: None,
    };
    Ok(doc)
}

fn default_input_path(pkg: &Package) -> Result<PathBuf> {
    for kind in &[TargetKind::Lib, TargetKind::Bin, TargetKind::ProcMacro] {
        for t in &pkg.targets {
            if t.kind.iter().any(|k| k == kind) {
                return Ok(t.src_path.clone());
            }
        }
    }
    Err(anyhow!(
        "failed to determine default source file for package `{}`",
        pkg.name
    ))
}

fn default_output_path(pkg: &Package) -> PathBuf {
    let base = pkg
        .readme
        .as_deref()
        .unwrap_or_else(|| Path::new("README.md"));
    pkg.manifest_path.parent().unwrap().join(base).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badges() {
        let config: Config = toml::from_str(
            r#"
[badges]
crates_io = false
docs_rs = false
github_workflow = { name = "ci" }
"#,
        )
        .unwrap();

        assert_eq!(
            config,
            Config {
                badges: Badges {
                    crates_io: false,
                    docs_rs: false,
                    github_workflow: Some(GitHubWorkflowBadge {
                        label: "ci".to_owned(),
                        name: "ci".to_owned(),
                    }),
                },
                docs: vec![],
                links: HashMap::new(),
            }
        );
    }

    #[test]
    fn single_input_string() {
        let config: Config = toml::from_str(
            r#"
[[ doc ]]
input = "src/lib.rs"
output = "README.md"
template = "docs/README_TEMPLATE.md"
"#,
        )
        .unwrap();

        assert_eq!(
            config,
            Config {
                badges: Badges::default(),
                docs: vec![Doc {
                    inputs: vec!["src/lib.rs".into()],
                    output: "README.md".into(),
                    template: Some("docs/README_TEMPLATE.md".into()),
                },],
                links: HashMap::new(),
            }
        );
    }

    #[test]
    fn multiple_input_strings() {
        let config: Config = toml::from_str(
            r#"
[[ doc ]]
input = ["src/lib.rs", "src/other.rs"]
output = "README.md"
template = "docs/README_TEMPLATE.md"
"#,
        )
        .unwrap();

        assert_eq!(
            config,
            Config {
                badges: Badges::default(),
                docs: vec![Doc {
                    inputs: vec!["src/lib.rs".into(), "src/other.rs".into()],
                    output: "README.md".into(),
                    template: Some("docs/README_TEMPLATE.md".into()),
                }],
                links: HashMap::new(),
            }
        );
    }
}
