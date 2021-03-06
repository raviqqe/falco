pub struct FilePathConverter {
    base_directory: std::path::PathBuf,
}

impl FilePathConverter {
    pub fn new(base_directory: impl AsRef<std::path::Path>) -> Self {
        Self {
            base_directory: base_directory.as_ref().into(),
        }
    }

    pub fn convert_to_os_path(&self, path: &app::FilePath) -> std::path::PathBuf {
        self.base_directory.join(
            path.components()
                .map(|component| component.replace("/", "_").replace("\\", "_"))
                .collect::<std::path::PathBuf>(),
        )
    }

    pub fn convert_absolute_to_file_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<app::FilePath, Box<dyn std::error::Error>> {
        Ok(app::FilePath::new(
            path.as_ref()
                .strip_prefix(&self.base_directory)
                .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "path outside package directory: {}",
                            path.as_ref().to_string_lossy()
                        ),
                    )
                })?
                .components()
                .filter_map(|component| match component {
                    std::path::Component::Normal(component) => {
                        Some(component.to_string_lossy().into())
                    }
                    _ => None,
                })
                .collect::<Vec<String>>(),
        ))
    }

    pub fn convert_relative_to_file_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<app::FilePath, Box<dyn std::error::Error>> {
        Ok(app::FilePath::new(
            path.as_ref()
                .components()
                .filter_map(|component| match component {
                    std::path::Component::Normal(component) => {
                        Some(component.to_string_lossy().into())
                    }
                    _ => None,
                })
                .collect::<Vec<String>>(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_empty_os_path() {
        let base_directory = &std::env::current_dir().unwrap();

        assert_eq!(
            &FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::FilePath::new(Vec::<&str>::new())),
            base_directory
        );
    }

    #[test]
    fn convert_to_os_path() {
        let base_directory = &std::env::current_dir().unwrap();

        assert_eq!(
            FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::FilePath::new(vec!["foo"])),
            base_directory.join("foo")
        );
    }

    #[test]
    fn convert_to_os_path_escaping_path() {
        let base_directory = &std::env::current_dir().unwrap();

        assert_eq!(
            FilePathConverter::new(base_directory)
                .convert_to_os_path(&app::FilePath::new(vec!["foo/bar\\baz"])),
            base_directory.join("foo_bar_baz")
        );
    }
}
