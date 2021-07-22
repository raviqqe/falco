use super::file_path_converter::FilePathConverter;

pub struct ExternalPackageDownloader<'a> {
    file_path_converter: &'a FilePathConverter,
}

impl<'a> ExternalPackageDownloader<'a> {
    pub fn new(file_path_converter: &'a FilePathConverter) -> Self {
        Self {
            file_path_converter,
        }
    }
}

impl<'a> app::ExternalPackageDownloader for ExternalPackageDownloader<'a> {
    fn download(
        &self,
        external_package: &app::ExternalPackage,
        directory_path: &app::FilePath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory_path = self.file_path_converter.convert_to_os_path(directory_path);

        let url = url::Url::parse(&["https://", external_package.url()].concat())?;
        let repository = git2::Repository::clone(url.as_str(), &directory_path)?;

        repository.checkout_tree(
            &repository
                .resolve_reference_from_short_name(external_package.version())?
                .peel(git2::ObjectType::Any)?,
            None,
        )?;

        Ok(())
    }
}
