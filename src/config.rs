use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Link remapping
    #[serde(default)]
    pub links: HashMap<String, String>,
}
