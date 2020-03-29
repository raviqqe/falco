use super::error::CompileError;
use super::type_compiler::TypeCompiler;
use crate::ast;

pub struct ExpressionCompiler<'a> {
    type_compiler: &'a TypeCompiler,
}

impl<'a> ExpressionCompiler<'a> {
    pub fn new(type_compiler: &'a TypeCompiler) -> Self {
        Self { type_compiler }
    }

    pub fn compile(
        &self,
        expression: &ast::Expression,
    ) -> Result<ssf::ir::Expression, CompileError> {
        match expression {
            ast::Expression::Application(application) => {
                let mut function = application.function();
                let mut arguments = vec![application.argument()];

                while let ast::Expression::Application(application) = &*function {
                    function = application.function();
                    arguments.push(application.argument());
                }

                Ok(ssf::ir::FunctionApplication::new(
                    self.compile(function)?
                        .to_variable()
                        .expect("variable")
                        .clone(),
                    arguments
                        .iter()
                        .rev()
                        .map(|argument| self.compile(argument))
                        .collect::<Result<_, _>>()?,
                )
                .into())
            }
            ast::Expression::Boolean(boolean) => Ok(ssf::ir::ConstructorApplication::new(
                ssf::ir::Constructor::new(
                    self.type_compiler.compile_boolean(),
                    boolean.value() as usize,
                ),
                vec![],
            )
            .into()),
            ast::Expression::Case(case) => {
                let argument = self.compile(case.argument())?;

                let alternatives = case
                    .alternatives()
                    .iter()
                    .take_while(|alternative| !alternative.pattern().is_variable())
                    .collect::<Vec<_>>();

                let default_alternative = case
                    .alternatives()
                    .iter()
                    .find(|alternative| alternative.pattern().is_variable())
                    .map(|alternative| -> Result<_, CompileError> {
                        Ok(ssf::ir::DefaultAlternative::new(
                            alternative.pattern().to_variable().unwrap().name(),
                            self.compile(alternative.expression())?,
                        ))
                    })
                    .transpose()?;

                match alternatives.get(0).map(|alternative| alternative.pattern()) {
                    Some(ast::Pattern::Boolean(_)) => Ok(ssf::ir::AlgebraicCase::new(
                        argument,
                        alternatives
                            .iter()
                            .map(|alternative| -> Result<_, CompileError> {
                                Ok(ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(
                                        self.type_compiler.compile_boolean(),
                                        alternative
                                            .pattern()
                                            .to_boolean()
                                            .map(|boolean| boolean.value())
                                            .unwrap()
                                            as usize,
                                    ),
                                    vec![],
                                    self.compile(alternative.expression())?,
                                ))
                            })
                            .collect::<Result<_, _>>()?,
                        default_alternative,
                    )
                    .into()),
                    Some(ast::Pattern::Number(_)) => Ok(ssf::ir::PrimitiveCase::new(
                        argument,
                        alternatives
                            .iter()
                            .map(|alternative| -> Result<_, CompileError> {
                                Ok(ssf::ir::PrimitiveAlternative::new(
                                    alternative
                                        .pattern()
                                        .to_number()
                                        .map(|number| number.value())
                                        .unwrap(),
                                    self.compile(alternative.expression())?,
                                ))
                            })
                            .collect::<Result<_, _>>()?,
                        default_alternative,
                    )
                    .into()),
                    Some(ast::Pattern::Record(_)) => unimplemented!(),
                    Some(ast::Pattern::None(_)) | Some(ast::Pattern::Variable(_)) | None => {
                        Err(CompileError::InvalidPattern(
                            case.alternatives()[0]
                                .pattern()
                                .source_information()
                                .clone(),
                        ))
                    }
                }
            }
            ast::Expression::If(_) => unreachable!(),
            ast::Expression::Let(let_) => match let_.definitions()[0] {
                ast::Definition::FunctionDefinition(_) => {
                    Ok(self.compile_let_functions(let_)?.into())
                }
                ast::Definition::ValueDefinition(_) => Ok(self.compile_let_values(let_)?.into()),
            },
            ast::Expression::None(_) => Ok(ssf::ir::ConstructorApplication::new(
                ssf::ir::Constructor::new(self.type_compiler.compile_none(), 0),
                vec![],
            )
            .into()),
            ast::Expression::Number(number) => {
                Ok(ssf::ir::Primitive::Float64(number.value()).into())
            }
            ast::Expression::Operation(operation) => Ok(ssf::ir::Operation::new(
                operation.operator().into(),
                self.compile(operation.lhs())?,
                self.compile(operation.rhs())?,
            )
            .into()),
            ast::Expression::Record(record) => Ok(ssf::ir::ConstructorApplication::new(
                ssf::ir::Constructor::new(self.type_compiler.compile_record(record.type_()), 0),
                record
                    .elements()
                    .iter()
                    .map(|(_, expression)| self.compile(expression))
                    .collect::<Result<_, _>>()?,
            )
            .into()),
            ast::Expression::Variable(variable) => Ok(ssf::ir::Expression::Variable(
                ssf::ir::Variable::new(variable.name()),
            )),
        }
    }

    fn compile_let_functions(
        &self,
        let_: &ast::Let,
    ) -> Result<ssf::ir::LetFunctions, CompileError> {
        let function_definitions = let_
            .definitions()
            .iter()
            .map(|definition| match definition {
                ast::Definition::FunctionDefinition(function_definition) => Ok(function_definition),
                ast::Definition::ValueDefinition(value_definition) => {
                    Err(CompileError::MixedDefinitionsInLet(
                        value_definition.source_information().clone(),
                    ))
                }
            })
            .collect::<Result<Vec<&ast::FunctionDefinition>, _>>()?;

        Ok(ssf::ir::LetFunctions::new(
            function_definitions
                .iter()
                .map(|function_definition| {
                    let type_ = function_definition
                        .type_()
                        .to_function()
                        .expect("function type");

                    Ok(ssf::ir::FunctionDefinition::new(
                        function_definition.name(),
                        function_definition
                            .arguments()
                            .iter()
                            .zip(type_.arguments())
                            .map(|(name, type_)| {
                                ssf::ir::Argument::new(
                                    name.clone(),
                                    self.type_compiler.compile(type_),
                                )
                            })
                            .collect(),
                        self.compile(function_definition.body())?,
                        self.type_compiler.compile_value(type_.last_result()),
                    ))
                })
                .collect::<Result<Vec<_>, CompileError>>()?,
            self.compile(let_.expression())?,
        ))
    }

    fn compile_let_values(&self, let_: &ast::Let) -> Result<ssf::ir::LetValues, CompileError> {
        let value_definitions = let_
            .definitions()
            .iter()
            .map(|definition| match definition {
                ast::Definition::FunctionDefinition(function_definition) => {
                    Err(CompileError::MixedDefinitionsInLet(
                        function_definition.source_information().clone(),
                    ))
                }
                ast::Definition::ValueDefinition(value_definition) => Ok(value_definition),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ssf::ir::LetValues::new(
            value_definitions
                .iter()
                .map(|value_definition| {
                    Ok(ssf::ir::ValueDefinition::new(
                        value_definition.name(),
                        self.compile(value_definition.body())?,
                        self.type_compiler.compile_value(value_definition.type_()),
                    ))
                })
                .collect::<Result<Vec<_>, CompileError>>()?,
            self.compile(let_.expression())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::type_compiler::TypeCompiler;
    use super::ExpressionCompiler;
    use crate::ast::*;
    use crate::debug::SourceInformation;
    use crate::types;
    use pretty_assertions::assert_eq;

    #[test]
    fn compile_operation() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Operation::new(
                    Operator::Add,
                    Number::new(1.0, SourceInformation::dummy()),
                    Number::new(2.0, SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(ssf::ir::Operation::new(ssf::ir::Operator::Add, 1.0, 2.0).into())
        );
    }

    #[test]
    fn compile_let_values() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Let::new(
                    vec![ValueDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy())
                )
                .into(),
            ),
            Ok(ssf::ir::LetValues::new(
                vec![ssf::ir::ValueDefinition::new(
                    "x",
                    42.0,
                    ssf::types::Primitive::Float64,
                )],
                ssf::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_functions() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy())
                )
                .into(),
            ),
            Ok(ssf::ir::LetFunctions::new(
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    42.0,
                    ssf::types::Primitive::Float64,
                )],
                ssf::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_functions_with_recursive_functions() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Application::new(
                            Variable::new("f", SourceInformation::dummy()),
                            Variable::new("x", SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy())
                )
                .into(),
            ),
            Ok(ssf::ir::LetFunctions::new(
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::FunctionApplication::new(
                        ssf::ir::Variable::new("f"),
                        vec![ssf::ir::Variable::new("x").into()]
                    ),
                    ssf::types::Primitive::Float64,
                )],
                ssf::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_nested_let_functions() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Let::new(
                            vec![FunctionDefinition::new(
                                "g",
                                vec!["y".into()],
                                Variable::new("x", SourceInformation::dummy()),
                                types::Function::new(
                                    types::Number::new(SourceInformation::dummy()),
                                    types::Number::new(SourceInformation::dummy()),
                                    SourceInformation::dummy()
                                ),
                                SourceInformation::dummy()
                            )
                            .into()],
                            Variable::new("x", SourceInformation::dummy())
                        ),
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy())
                )
                .into(),
            ),
            Ok(ssf::ir::LetFunctions::new(
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetFunctions::new(
                        vec![ssf::ir::FunctionDefinition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        )],
                        ssf::ir::Variable::new("x")
                    ),
                    ssf::types::Primitive::Float64,
                )],
                ssf::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_values_with_free_variables() {
        assert_eq!(
            ExpressionCompiler::new(&TypeCompiler::new(&Module::dummy())).compile(
                &Let::new(
                    vec![ValueDefinition::new(
                        "y",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Let::new(
                        vec![FunctionDefinition::new(
                            "f",
                            vec!["x".into()],
                            Variable::new("y", SourceInformation::dummy()),
                            types::Function::new(
                                types::Number::new(SourceInformation::dummy()),
                                types::Number::new(SourceInformation::dummy()),
                                SourceInformation::dummy()
                            ),
                            SourceInformation::dummy()
                        )
                        .into()],
                        Variable::new("y", SourceInformation::dummy())
                    )
                )
                .into(),
            ),
            Ok(ssf::ir::LetValues::new(
                vec![ssf::ir::ValueDefinition::new(
                    "y",
                    42.0,
                    ssf::types::Primitive::Float64,
                )],
                ssf::ir::LetFunctions::new(
                    vec![ssf::ir::FunctionDefinition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Variable::new("y"),
                        ssf::types::Primitive::Float64,
                    )],
                    ssf::ir::Variable::new("y")
                )
            )
            .into())
        );
    }

    #[test]
    fn compile_case_expressions() {
        let type_compiler = TypeCompiler::new(&Module::dummy());
        let boolean_type = type_compiler.compile_boolean();

        for (source, target) in vec![
            (
                Case::new(
                    Number::new(42.0, SourceInformation::dummy()),
                    vec![Alternative::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        Number::new(42.0, SourceInformation::dummy()),
                    )],
                    SourceInformation::dummy(),
                ),
                ssf::ir::PrimitiveCase::new(
                    42.0,
                    vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                    None,
                )
                .into(),
            ),
            (
                Case::new(
                    Number::new(42.0, SourceInformation::dummy()),
                    vec![
                        Alternative::new(
                            Number::new(42.0, SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                        ),
                        Alternative::new(
                            Variable::new("x", SourceInformation::dummy()),
                            Variable::new("x", SourceInformation::dummy()),
                        ),
                    ],
                    SourceInformation::dummy(),
                ),
                ssf::ir::PrimitiveCase::new(
                    42.0,
                    vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                    Some(ssf::ir::DefaultAlternative::new(
                        "x",
                        ssf::ir::Variable::new("x"),
                    )),
                )
                .into(),
            ),
            (
                Case::new(
                    Boolean::new(false, SourceInformation::dummy()),
                    vec![
                        Alternative::new(
                            Boolean::new(false, SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                        ),
                        Alternative::new(
                            Boolean::new(true, SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                        ),
                    ],
                    SourceInformation::dummy(),
                ),
                ssf::ir::AlgebraicCase::new(
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(boolean_type.clone(), 0),
                        vec![],
                    ),
                    vec![
                        ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(boolean_type.clone(), 0),
                            vec![],
                            42.0,
                        ),
                        ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(boolean_type.clone(), 1),
                            vec![],
                            42.0,
                        ),
                    ],
                    None,
                )
                .into(),
            ),
            (
                Case::new(
                    Boolean::new(false, SourceInformation::dummy()),
                    vec![
                        Alternative::new(
                            Boolean::new(false, SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                        ),
                        Alternative::new(
                            Variable::new("x", SourceInformation::dummy()),
                            Variable::new("x", SourceInformation::dummy()),
                        ),
                    ],
                    SourceInformation::dummy(),
                ),
                ssf::ir::AlgebraicCase::new(
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(boolean_type.clone(), 0),
                        vec![],
                    ),
                    vec![ssf::ir::AlgebraicAlternative::new(
                        ssf::ir::Constructor::new(boolean_type.clone(), 0),
                        vec![],
                        42.0,
                    )],
                    Some(ssf::ir::DefaultAlternative::new(
                        "x",
                        ssf::ir::Variable::new("x"),
                    )),
                )
                .into(),
            ),
        ] {
            assert_eq!(
                ExpressionCompiler::new(&type_compiler).compile(&source.into()),
                Ok(target)
            );
        }
    }

    #[test]
    fn compile_records() {
        let type_ = types::Record::new(
            "Foo",
            vec![(
                "foo".into(),
                types::Number::new(SourceInformation::dummy()).into(),
            )]
            .into_iter()
            .collect(),
            SourceInformation::dummy(),
        );
        let type_compiler = TypeCompiler::new(&Module::from_definitions_and_type_definitions(
            vec![TypeDefinition::new("Foo", type_.clone())],
            vec![],
        ));

        assert_eq!(
            ExpressionCompiler::new(&type_compiler).compile(
                &Record::new(
                    type_,
                    vec![(
                        "foo".into(),
                        Number::new(42.0, SourceInformation::dummy()).into()
                    )]
                    .into_iter()
                    .collect(),
                    SourceInformation::dummy(),
                )
                .into(),
            ),
            Ok(ssf::ir::ConstructorApplication::new(
                ssf::ir::Constructor::new(
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                        ssf::types::Primitive::Float64.into()
                    ])]),
                    0
                ),
                vec![ssf::ir::Primitive::Float64(42.0).into()]
            )
            .into())
        );
    }
}
