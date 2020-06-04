pub struct CompileConfiguration {
    source_main_function_name: String,
    object_main_function_name: String,
    object_init_function_name: String,
}

impl CompileConfiguration {
    pub fn new(
        source_main_function_name: impl Into<String>,
        object_main_function_name: impl Into<String>,
        object_init_function_name: impl Into<String>,
    ) -> CompileConfiguration {
        Self {
            source_main_function_name: source_main_function_name.into(),
            object_main_function_name: object_main_function_name.into(),
            object_init_function_name: object_init_function_name.into(),
        }
    }

    pub fn source_main_function_name(&self) -> &str {
        &self.source_main_function_name
    }

    pub fn object_main_function_name(&self) -> &str {
        &self.object_main_function_name
    }

    pub fn object_init_function_name(&self) -> &str {
        &self.object_init_function_name
    }
}