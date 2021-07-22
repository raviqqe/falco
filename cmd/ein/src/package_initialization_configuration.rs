use once_cell::sync::Lazy;

pub static DEFAULT_SYSTEM_PACKAGE_CONFIGURATION: Lazy<app::ExternalPackage> =
    Lazy::new(|| app::ExternalPackage::new("Os", "github.com/ein-lang/os", "main"));

pub static PACKAGE_INITIALIZATION_CONFIGURATION: app::PackageInitializationConfiguration =
    app::PackageInitializationConfiguration {
        application_main_file_content: indoc::indoc!(
            "
            import Os.Os

            main : Os.Os -> Number
            main os =
              let
                result = Os.fdWrite os Os.stdout \"Hello, world!\\n\"
              in
                case _ = result
                  Number => 0
                  Error => 1
            "
        ),
        library_main_file_content: indoc::indoc!(
            "
            export { foo }

            foo : Number -> Number
            foo x = x
            "
        ),
        library_main_basename: "Foo",
    };
