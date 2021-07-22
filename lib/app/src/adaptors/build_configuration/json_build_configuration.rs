use super::{
    json_application_build_configuration::JsonApplicationBuildConfiguration,
    json_external_package_configuration::JsonExternalPackageConfiguration,
    json_system_package_configuration::JsonSystemPackageConfiguration,
};
use crate::{
    common::{ApplicationTarget, BuildConfiguration},
    ExternalPackage, Target,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonBuildConfiguration {
    application: Option<JsonApplicationBuildConfiguration>,
    dependencies: HashMap<String, JsonExternalPackageConfiguration>,
}

impl JsonBuildConfiguration {
    pub fn new(
        application: Option<JsonApplicationBuildConfiguration>,
        dependencies: HashMap<String, JsonExternalPackageConfiguration>,
    ) -> Self {
        Self {
            application,
            dependencies,
        }
    }

    pub fn serialize(configuration: &BuildConfiguration) -> Self {
        JsonBuildConfiguration::new(
            match configuration.target() {
                Target::Application(application) => Some(JsonApplicationBuildConfiguration::new(
                    application.name(),
                    JsonSystemPackageConfiguration::new(
                        application.system_package().name(),
                        application.system_package().url(),
                        application.system_package().version(),
                    ),
                )),
                Target::Library => None,
            },
            configuration
                .dependencies()
                .iter()
                .map(|external_package| {
                    (
                        external_package.name().into(),
                        JsonExternalPackageConfiguration::new(
                            external_package.url(),
                            external_package.version(),
                        ),
                    )
                })
                .collect(),
        )
    }

    pub fn deserialize(&self) -> BuildConfiguration {
        BuildConfiguration::new(
            self.application
                .as_ref()
                .map(|application| {
                    ApplicationTarget::new(
                        application.name(),
                        ExternalPackage::new(
                            application.system().name(),
                            application.system().url(),
                            application.system().version(),
                        ),
                    )
                    .into()
                })
                .unwrap_or(Target::Library),
            self.dependencies
                .iter()
                .map(|(name, configuration)| {
                    ExternalPackage::new(name, configuration.url(), configuration.version())
                })
                .collect(),
        )
    }
}
