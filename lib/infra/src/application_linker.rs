use super::{command_runner::CommandRunner, file_path_converter::FilePathConverter};

pub struct ApplicationLinker<'a> {
    command_runner: &'a CommandRunner,
    file_path_converter: &'a FilePathConverter,
}

impl<'a> ApplicationLinker<'a> {
    pub fn new(
        command_runner: &'a CommandRunner,
        file_path_converter: &'a FilePathConverter,
    ) -> Self {
        Self {
            command_runner,
            file_path_converter,
        }
    }
}

impl<'a> app::ApplicationLinker for ApplicationLinker<'a> {
    fn link(
        &self,
        object_file_paths: &[app::FilePath],
        application_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bitcode_paths, ffi_paths) = object_file_paths
            .iter()
            .map(|path| self.file_path_converter.convert_to_os_path(path))
            .partition::<Vec<_>, _>(|path| path.extension() == Some(std::ffi::OsStr::new("bc")));
        let llc = which::which("llc-13")
            .or_else(|_| which::which("llc-12"))
            .or_else(|_| which::which("llc-11"))
            .or_else(|_| which::which("llc"))?;

        for path in &bitcode_paths {
            // LLVM C API doesn't seem to support the tailcallopt pass directly.
            // So we compile each bitcode file with the pass manually in order
            // to optimize all tail calls.
            self.command_runner.run(
                std::process::Command::new(&llc)
                    .arg("-O3")
                    .arg("-tailcallopt")
                    .arg("--relocation-model=pic")
                    .arg("-filetype=obj")
                    .arg(path),
            )?;
        }

        self.command_runner.run(
            std::process::Command::new("clang")
                .arg("-Werror") // cspell:disable-line
                .arg("-Wno-incompatible-pointer-types-discards-qualifiers") // cspell:disable-line
                .arg("-Wno-override-module") // cspell:disable-line
                .arg("-o")
                .arg(
                    self.file_path_converter
                        .convert_to_os_path(&app::FilePath::new(&[application_name])),
                )
                .arg("-O3")
                .args(bitcode_paths.iter().map(|path| {
                    let mut path = path.clone();
                    path.set_extension("o");
                    path
                }))
                .args(ffi_paths)
                .arg("-ldl")
                .arg("-lpthread"),
        )?;

        Ok(())
    }
}
