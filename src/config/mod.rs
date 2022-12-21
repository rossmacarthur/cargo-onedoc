mod input;

use std::collections::HashMap;
use std::fs;
use std::io;

use anyhow::{anyhow, Context as _, Result};
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use cargo_metadata::{Metadata, Package};
use serde::Deserialize;

/// Configuration of which files to process.
#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Config {
    /// A list of processes that each outputs a single Markdown file
    #[serde(default, rename = "doc")]
    pub docs: Vec<Doc>,

    /// Global link remapping config
    #[serde(default)]
    pub links: HashMap<String, String>,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
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
    for kind in ["lib", "bin", "proc-macro"] {
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
