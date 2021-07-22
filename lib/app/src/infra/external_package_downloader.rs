use crate::common::{ExternalPackage, FilePath};

pub trait ExternalPackageDownloader {
    fn download(
        &self,
        external_package: &ExternalPackage,
        directory_path: &FilePath,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(test)]
pub struct FakeExternalPackageDownloader<'a, S: crate::infra::FileSystem> {
    packages: std::collections::HashMap<String, std::collections::HashMap<FilePath, Vec<u8>>>,
    file_system: &'a S,
}

#[cfg(test)]
impl<'a, S: crate::infra::FileSystem> FakeExternalPackageDownloader<'a, S> {
    pub fn new(
        packages: std::collections::HashMap<String, std::collections::HashMap<FilePath, Vec<u8>>>,
        file_system: &'a S,
    ) -> Self {
        Self {
            packages,
            file_system,
        }
    }
}

#[cfg(test)]
impl<'a, S: crate::infra::FileSystem> ExternalPackageDownloader
    for FakeExternalPackageDownloader<'a, S>
{
    fn download(
        &self,
        external_package: &ExternalPackage,
        directory_path: &FilePath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (path, data) in &self.packages[external_package.url()] {
            self.file_system.write(&directory_path.join(path), data)?;
        }

        Ok(())
    }
}
