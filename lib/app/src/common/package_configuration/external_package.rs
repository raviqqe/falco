#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExternalPackage {
    name: String,
    url: String,
    // This version is the "expected" one different from lang::Package.
    version: String,
}

impl ExternalPackage {
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
