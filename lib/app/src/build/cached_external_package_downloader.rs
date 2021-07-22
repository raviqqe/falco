use super::package_configuration_reader::PackageConfigurationReader;
use crate::{
    common::{ExternalPackage, FilePathResolver, PackageConfiguration},
    infra::{ExternalPackageDownloader, FileSystem, Logger},
};

pub struct CachedExternalPackageDownloader<'a> {
    package_configuration_reader: &'a PackageConfigurationReader<'a>,
    external_package_downloader: &'a dyn ExternalPackageDownloader,
    file_system: &'a dyn FileSystem,
    file_path_resolver: &'a FilePathResolver<'a>,
    logger: &'a dyn Logger,
}

impl<'a> CachedExternalPackageDownloader<'a> {
    pub fn new(
        package_configuration_reader: &'a PackageConfigurationReader<'a>,
        external_package_downloader: &'a dyn ExternalPackageDownloader,
        file_system: &'a dyn FileSystem,
        file_path_resolver: &'a FilePathResolver<'a>,
        logger: &'a dyn Logger,
    ) -> Self {
        Self {
            package_configuration_reader,
            external_package_downloader,
            file_system,
            file_path_resolver,
            logger,
        }
    }

    pub fn download(
        &self,
        external_package: &ExternalPackage,
    ) -> Result<PackageConfiguration, Box<dyn std::error::Error>> {
        let directory_path = self
            .file_path_resolver
            .resolve_external_package_directory_path(external_package);

        if !self.file_system.exists(&directory_path) {
            self.logger.log(&format!(
                "downloading package {} {} {}",
                external_package.name(),
                external_package.url(),
                external_package.version()
            ))?;

            self.external_package_downloader
                .download(external_package, &directory_path)?;
        }

        self.package_configuration_reader.read(&directory_path)
    }
}
