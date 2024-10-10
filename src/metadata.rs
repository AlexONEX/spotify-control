use std::collections::HashMap;
use std::fmt::Display;
use zbus::zvariant::{Array, OwnedValue};

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub artwork: String,
}

#[derive(Debug, Clone)]
pub enum MetadataError {
    MissingKey(String),
    #[allow(dead_code)]
    InvalidValueType(String),
}

impl std::error::Error for MetadataError {}

impl Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::MissingKey(key) => write!(f, "Missing metadata key: {}", key),
            MetadataError::InvalidValueType(key) => {
                write!(f, "Invalid value type for key: {}", key)
            }
        }
    }
}

impl TryFrom<HashMap<String, OwnedValue>> for Metadata {
    type Error = MetadataError;

    fn try_from(map: HashMap<String, OwnedValue>) -> Result<Self, Self::Error> {
        Ok(Metadata {
            title: get_string(&map, "xesam:title")?,
            artists: get_string_vec(&map, "xesam:artist")?,
            album: get_string(&map, "xesam:album")?,
            artwork: get_string(&map, "mpris:artUrl")?,
        })
    }
}

fn get_string(map: &HashMap<String, OwnedValue>, key: &str) -> Result<String, MetadataError> {
    map.get(key)
        .and_then(|v| v.downcast_ref::<str>().map(|s| s.to_string()))
        .ok_or_else(|| MetadataError::MissingKey(key.to_string()))
}

fn get_string_vec(
    map: &HashMap<String, OwnedValue>,
    key: &str,
) -> Result<Vec<String>, MetadataError> {
    map.get(key)
        .and_then(|v| v.downcast_ref::<Array>())
        .map(|array| {
            array
                .get()
                .iter()
                .filter_map(|v| v.downcast_ref::<str>().map(|s| s.to_string()))
                .collect()
        })
        .ok_or_else(|| MetadataError::MissingKey(key.to_string()))
}
