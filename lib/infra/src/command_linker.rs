use super::command_runner::CommandRunner;
use super::file_path_converter::FilePathConverter;

pub struct CommandLinker<'a> {
    file_path_converter: &'a FilePathConverter,
    runtime_library_path: std::path::PathBuf,
}

impl<'a> CommandLinker<'a> {
    pub fn new(
        file_path_converter: &'a FilePathConverter,
        runtime_library_path: impl AsRef<std::path::Path>,
    ) -> Self {
        Self {
            file_path_converter,
            runtime_library_path: runtime_library_path.as_ref().into(),
        }
    }
}

impl<'a> app::CommandLinker for CommandLinker<'a> {
    fn link(
        &self,
        object_file_paths: &[app::FilePath],
        command_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        CommandRunner::run(
            std::process::Command::new("clang")
                .arg("-o")
                .arg(command_name)
                .arg("-O3")
                .arg("-flto")
                .arg("-ldl")
                .arg("-lpthread")
                .args(
                    object_file_paths
                        .iter()
                        .map(|path| self.file_path_converter.convert_to_os_path(path)),
                )
                .arg(&self.runtime_library_path),
        )
    }
}
