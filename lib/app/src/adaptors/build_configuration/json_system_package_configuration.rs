use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct JsonSystemPackageConfiguration {
    name: String,
    url: String,
    version: String,
}

impl JsonSystemPackageConfiguration {
    pub fn new(
        name: impl Into<String>,
        url: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            version: version.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
