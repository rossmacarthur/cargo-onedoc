use std::collections::HashMap;

use camino::Utf8PathBuf as PathBuf;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Files to process
    #[serde(default)]
    pub files: Vec<File>,

    /// Link remapping
    #[serde(default)]
    pub links: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct File {
    pub input: PathBuf,
    pub output: PathBuf,
    pub template: Option<PathBuf>,
}
