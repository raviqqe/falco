use crate::common::{ExternalPackage, FilePath};

#[derive(Debug, PartialEq)]
pub enum BuildError {
    ExternalPackageConfigurationFileNotFound {
        package_name: String,
    },
    MainFunctionModuleNotFound {
        main_function_module_name: &'static str,
        external_package: ExternalPackage,
    },
    ModuleCircularDependency(FilePath),
    ModuleNotFound {
        module_path: lang::UnresolvedModulePath,
        source_file_path: FilePath,
    },
    PackageCircularDependency(ExternalPackage),
}

impl std::error::Error for BuildError {}

impl std::fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ExternalPackageConfigurationFileNotFound { package_name } => write!(
                formatter,
                "package configuration file not found in external package \"{}\"",
                package_name
            ),
            Self::MainFunctionModuleNotFound {
                main_function_module_name,
                external_package,
            } => write!(
                formatter,
                "\"{}\" module not found in system package \"{}\"",
                &main_function_module_name,
                external_package.name(),
            ),
            Self::ModuleCircularDependency(file_path) => write!(
                formatter,
                "circular module dependency detected: {}",
                file_path
            ),
            Self::ModuleNotFound {
                module_path,
                source_file_path,
            } => {
                write!(
                    formatter,
                    "module \"{}\" not found imported by \"{}\"",
                    module_path, source_file_path
                )
            }
            Self::PackageCircularDependency(external_package) => write!(
                formatter,
                "circular package dependency detected: {}",
                external_package.name(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_external_package_configuration_not_found_error() {
        assert_eq!(
            format!(
                "{}",
                BuildError::ExternalPackageConfigurationFileNotFound {
                    package_name: "foo".into()
                }
            ),
            "package configuration file not found in external package \"foo\""
        );
    }

    #[test]
    fn display_module_circular_dependency() {
        assert_eq!(
            format!(
                "{}",
                BuildError::ModuleCircularDependency(FilePath::new(vec!["foo", "bar", "baz"]))
            ),
            "circular module dependency detected: foo/bar/baz"
        );
    }

    #[test]
    fn display_package_circular_dependency() {
        assert_eq!(
            format!(
                "{}",
                BuildError::PackageCircularDependency(ExternalPackage::new("foo", "1.2.3"))
            ),
            "circular package dependency detected: foo 1.2.3"
        );
    }
}
