use super::{
    file_path::FilePath,
    file_path_configuration::{
        FilePathConfiguration, INTERFACE_FILE_EXTENSION, OBJECT_FILE_EXTENSION,
    },
    package_configuration::ExternalPackage,
    static_file_path_manager::StaticFilePathManager,
};

pub struct FilePathResolver<'a> {
    static_file_path_manager: &'a StaticFilePathManager,
    file_path_configuration: &'a FilePathConfiguration,
}

impl<'a> FilePathResolver<'a> {
    pub fn new(
        static_file_path_manager: &'a StaticFilePathManager,
        file_path_configuration: &'a FilePathConfiguration,
    ) -> Self {
        Self {
            static_file_path_manager,
            file_path_configuration,
        }
    }

    pub fn resolve_source_file_path(
        &self,
        directory_path: &FilePath,
        internal_module_path: &lang::InternalUnresolvedModulePath,
    ) -> FilePath {
        directory_path.join(
            &FilePath::new(internal_module_path.components())
                .with_extension(self.file_path_configuration.source_file_extension),
        )
    }

    pub fn resolve_object_file_path(&self, id: impl AsRef<str>) -> FilePath {
        self.static_file_path_manager
            .object_directory_path()
            .join(&FilePath::new(&[&id]))
            .with_extension(OBJECT_FILE_EXTENSION)
    }

    pub fn resolve_interface_file_path(&self, id: &str) -> FilePath {
        self.resolve_object_file_path(id)
            .with_extension(INTERFACE_FILE_EXTENSION)
    }

    pub fn resolve_module_path(
        &self,
        source_file_path: &FilePath,
        package: &lang::Package,
    ) -> lang::ModulePath {
        lang::ModulePath::new(
            package.clone(),
            source_file_path
                .with_extension("")
                .components()
                .map(String::from)
                .collect(),
        )
    }

    pub fn resolve_external_package_directory_path(
        &self,
        external_package: &ExternalPackage,
    ) -> FilePath {
        self.static_file_path_manager
            .external_packages_directory_path()
            .join(
                &FilePath::new(&[external_package.name()])
                    .join(&FilePath::new(&[external_package.version()])),
            )
    }
}
