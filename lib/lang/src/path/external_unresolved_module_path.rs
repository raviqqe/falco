use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ExternalUnresolvedModulePath {
    package_name: String,
    components: Vec<String>,
}

impl ExternalUnresolvedModulePath {
    pub fn new(package_name: impl Into<String>, components: Vec<String>) -> Self {
        Self {
            package_name: package_name.into(),
            components,
        }
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.components.iter().map(Deref::deref)
    }
}

impl std::fmt::Display for ExternalUnresolvedModulePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.components.join("/"))
    }
}
