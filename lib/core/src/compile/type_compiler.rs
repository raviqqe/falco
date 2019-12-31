use super::llvm;
use crate::ast;
use crate::types::{self, Type};

pub struct TypeCompiler {}

impl TypeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, type_: &Type) -> llvm::Type {
        match type_ {
            Type::Function(function) => llvm::Type::pointer(self.compile_unsized_closure(function)),
            Type::Value(value) => self.compile_value(value),
        }
    }

    pub fn compile_value(&self, value: &types::Value) -> llvm::Type {
        match value {
            types::Value::Number => llvm::Type::double(),
        }
    }

    pub fn compile_entry_function(&self, function: &types::Function) -> llvm::Type {
        let mut arguments = vec![llvm::Type::pointer(self.compile_unsized_environment())];

        arguments.extend_from_slice(
            &function
                .arguments()
                .iter()
                .map(|type_| self.compile(type_))
                .collect::<Vec<_>>(),
        );

        llvm::Type::function(self.compile_value(function.result()), &arguments)
    }

    pub fn compile_closure(&self, function_definition: &ast::FunctionDefinition) -> llvm::Type {
        llvm::Type::struct_(&[
            llvm::Type::pointer(self.compile_entry_function(function_definition.type_())),
            self.compile_environment(function_definition.environment()),
        ])
    }

    pub fn compile_unsized_closure(&self, function: &types::Function) -> llvm::Type {
        llvm::Type::struct_(&[
            llvm::Type::pointer(self.compile_entry_function(function)),
            self.compile_unsized_environment(),
        ])
    }

    pub fn compile_environment(&self, free_variables: &[ast::Argument]) -> llvm::Type {
        llvm::Type::struct_(
            &free_variables
                .iter()
                .map(|argument| self.compile(argument.type_()))
                .collect::<Vec<_>>(),
        )
    }

    fn compile_unsized_environment(&self) -> llvm::Type {
        llvm::Type::struct_(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_number() {
        TypeCompiler::new().compile(&types::Value::Number.into());
    }

    #[test]
    fn compile_function() {
        TypeCompiler::new().compile(
            &types::Function::new(vec![types::Value::Number.into()], types::Value::Number).into(),
        );
    }

    #[test]
    fn compile_function_twice() {
        let compiler = TypeCompiler::new();
        let type_ =
            types::Function::new(vec![types::Value::Number.into()], types::Value::Number).into();

        compiler.compile(&type_);
        compiler.compile(&type_);
    }
}
