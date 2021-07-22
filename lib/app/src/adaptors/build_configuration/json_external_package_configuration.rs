use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct JsonExternalPackageConfiguration {
    url: String,
    version: String,
}

impl JsonExternalPackageConfiguration {
    pub fn new(url: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            version: version.into(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
