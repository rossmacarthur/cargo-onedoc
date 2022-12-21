use super::*;

use std::fmt;

use serde::de;
use serde::de::{Deserialize, Deserializer};

struct Visitor;

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Vec<PathBuf>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("string or map")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let d = de::value::StrDeserializer::new(v);
        let path: PathBuf = Deserialize::deserialize(d)?;
        Ok(vec![path])
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let d = de::value::SeqAccessDeserializer::new(seq);
        Deserialize::deserialize(d)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(Visitor)
}
