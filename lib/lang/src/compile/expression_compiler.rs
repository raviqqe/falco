use super::{
    error::CompileError,
    last_result_type_calculator::LastResultTypeCalculator,
    reference_type_resolver::ReferenceTypeResolver,
    string_type_configuration::StringTypeConfiguration,
    transform::{
        BooleanOperationTransformer, EqualOperationTransformer, FunctionTypeCoercionTransformer,
        LetErrorTransformer, ListCaseTransformer, ListLiteralTransformer,
        ListTypeCoercionTransformer, NotEqualOperationTransformer,
    },
    type_compiler::TypeCompiler,
    variable_compiler::VariableCompiler,
};
use crate::{ast::*, types::Type};
use std::sync::Arc;

pub struct ExpressionCompilerSet {
    pub variable_compiler: Arc<VariableCompiler>,
}

pub struct ExpressionTransformerSet {
    pub equal_operation_transformer: Arc<EqualOperationTransformer>,
    pub not_equal_operation_transformer: Arc<NotEqualOperationTransformer>,
    pub list_literal_transformer: Arc<ListLiteralTransformer>,
    pub boolean_operation_transformer: Arc<BooleanOperationTransformer>,
    pub function_type_coercion_transformer: Arc<FunctionTypeCoercionTransformer>,
    pub list_type_coercion_transformer: Arc<ListTypeCoercionTransformer>,
    pub list_case_transformer: Arc<ListCaseTransformer>,
    pub let_error_transformer: Arc<LetErrorTransformer>,
}

pub struct ExpressionCompiler {
    expression_compiler_set: Arc<ExpressionCompilerSet>,
    expression_transformer_set: Arc<ExpressionTransformerSet>,
    reference_type_resolver: Arc<ReferenceTypeResolver>,
    last_result_type_calculator: Arc<LastResultTypeCalculator>,
    type_compiler: Arc<TypeCompiler>,
    string_type_configuration: Arc<StringTypeConfiguration>,
}

impl ExpressionCompiler {
    pub fn new(
        expression_compiler_set: Arc<ExpressionCompilerSet>,
        expression_transformer_set: Arc<ExpressionTransformerSet>,
        reference_type_resolver: Arc<ReferenceTypeResolver>,
        last_result_type_calculator: Arc<LastResultTypeCalculator>,
        type_compiler: Arc<TypeCompiler>,
        string_type_configuration: Arc<StringTypeConfiguration>,
    ) -> Arc<Self> {
        Self {
            expression_compiler_set,
            expression_transformer_set,
            reference_type_resolver,
            last_result_type_calculator,
            type_compiler,
            string_type_configuration,
        }
        .into()
    }

    pub fn compile(&self, expression: &Expression) -> Result<eir::ir::Expression, CompileError> {
        Ok(match expression {
            Expression::Application(application) => eir::ir::FunctionApplication::new(
                self.type_compiler
                    .compile(application.type_())?
                    .into_function()
                    .unwrap(),
                self.compile(application.function())?,
                self.compile(application.argument())?,
            )
            .into(),
            Expression::Boolean(boolean) => boolean.value().into(),
            Expression::Case(case) => self.compile_case(case)?,
            Expression::If(if_) => eir::ir::If::new(
                self.compile(if_.condition())?,
                self.compile(if_.then())?,
                self.compile(if_.else_())?,
            )
            .into(),
            Expression::Let(let_) => self.compile_let(let_)?,
            Expression::LetError(let_) => self.compile_let_error(let_)?,
            Expression::None(_) => {
                eir::ir::Record::new(self.type_compiler.compile_none(), vec![]).into()
            }
            Expression::List(list) => self.compile(
                &self
                    .expression_transformer_set
                    .list_literal_transformer
                    .transform(list)?,
            )?,
            Expression::ListCase(case) => self.compile(
                &self
                    .expression_transformer_set
                    .list_case_transformer
                    .transform(case)?,
            )?,
            Expression::Number(number) => number.value().into(),
            Expression::Operation(operation) => match operation {
                Operation::Arithmetic(operation) => eir::ir::ArithmeticOperation::new(
                    Self::compile_arithmetic_operator(operation.operator()),
                    self.compile(operation.lhs())?,
                    self.compile(operation.rhs())?,
                )
                .into(),
                Operation::Boolean(operation) => self.compile(
                    &self
                        .expression_transformer_set
                        .boolean_operation_transformer
                        .transform(operation),
                )?,
                Operation::Equality(operation) => match operation.operator() {
                    EqualityOperator::Equal => {
                        match self.reference_type_resolver.resolve(operation.type_())? {
                            Type::Number(_) => eir::ir::ComparisonOperation::new(
                                eir::ir::ComparisonOperator::Equal,
                                self.compile(operation.lhs())?,
                                self.compile(operation.rhs())?,
                            )
                            .into(),
                            Type::String(_) => eir::ir::FunctionApplication::new(
                                eir::types::Function::new(
                                    eir::types::Type::ByteString,
                                    eir::types::Type::Boolean,
                                ),
                                eir::ir::FunctionApplication::new(
                                    eir::types::Function::new(
                                        eir::types::Type::ByteString,
                                        eir::types::Function::new(
                                            eir::types::Type::ByteString,
                                            eir::types::Type::Boolean,
                                        ),
                                    ),
                                    eir::ir::Variable::new(
                                        &self.string_type_configuration.equal_function_name,
                                    ),
                                    self.compile(operation.lhs())?,
                                ),
                                self.compile(operation.rhs())?,
                            )
                            .into(),
                            _ => self.compile(
                                &self
                                    .expression_transformer_set
                                    .equal_operation_transformer
                                    .transform(operation)?,
                            )?,
                        }
                    }
                    EqualityOperator::NotEqual => self.compile(
                        &self
                            .expression_transformer_set
                            .not_equal_operation_transformer
                            .transform(operation),
                    )?,
                },
                Operation::Order(operation) => eir::ir::ComparisonOperation::new(
                    Self::compile_order_operator(operation.operator()),
                    self.compile(operation.lhs())?,
                    self.compile(operation.rhs())?,
                )
                .into(),
                Operation::Pipe(operation) => self.compile(
                    &Application::with_type(
                        operation.type_().clone(),
                        operation.rhs().clone(),
                        operation.lhs().clone(),
                        operation.source_information().clone(),
                    )
                    .into(),
                )?,
            },
            Expression::RecordConstruction(record) => eir::ir::Record::new(
                self.type_compiler
                    .compile(record.type_())?
                    .into_record()
                    .unwrap(),
                self.reference_type_resolver
                    .resolve_to_record(record.type_())?
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|element| self.compile(&record.elements()[element.name()]))
                    .collect::<Result<_, _>>()?,
            )
            .into(),
            Expression::RecordElementOperation(operation) => eir::ir::RecordElement::new(
                self.type_compiler
                    .compile(operation.type_())?
                    .into_record()
                    .unwrap(),
                self.reference_type_resolver
                    .resolve_to_record(operation.type_())?
                    .unwrap()
                    .elements()
                    .iter()
                    .position(|element| element.name() == operation.element_name())
                    .unwrap(),
                self.compile(operation.argument())?,
            )
            .into(),
            Expression::String(string) => eir::ir::ByteString::new(string.value()).into(),
            Expression::TypeCoercion(coercion) => {
                if self.reference_type_resolver.is_list(coercion.from())?
                    && self.reference_type_resolver.is_list(coercion.to())?
                {
                    self.compile(
                        &self
                            .expression_transformer_set
                            .list_type_coercion_transformer
                            .transform(coercion)?,
                    )?
                } else if self.reference_type_resolver.is_function(coercion.from())?
                    && self.reference_type_resolver.is_function(coercion.to())?
                {
                    self.compile(
                        &self
                            .expression_transformer_set
                            .function_type_coercion_transformer
                            .transform(coercion)?,
                    )?
                } else if self.reference_type_resolver.is_union(coercion.from())?
                    && (self.reference_type_resolver.is_function(coercion.to())?
                        || self.reference_type_resolver.is_list(coercion.to())?)
                {
                    let union_type = self
                        .reference_type_resolver
                        .resolve_to_union(coercion.from())?
                        .unwrap();
                    let source_information = coercion.source_information();
                    let argument_name = "$arg";

                    self.compile(
                        &Case::with_type(
                            union_type.clone(),
                            argument_name,
                            coercion.argument().clone(),
                            union_type
                                .types()
                                .iter()
                                .map(|type_| {
                                    Alternative::new(
                                        type_.clone(),
                                        TypeCoercion::new(
                                            Variable::new(
                                                argument_name,
                                                source_information.clone(),
                                            ),
                                            type_.clone(),
                                            coercion.to().clone(),
                                            source_information.clone(),
                                        ),
                                    )
                                })
                                .collect(),
                            source_information.clone(),
                        )
                        .into(),
                    )?
                } else {
                    // Coerce to union or Any types.
                    let from_type = self.reference_type_resolver.resolve(coercion.from())?;
                    let argument = self.compile(coercion.argument())?;

                    match &from_type {
                        Type::Boolean(_)
                        | Type::Function(_)
                        | Type::None(_)
                        | Type::Number(_)
                        | Type::Record(_)
                        | Type::String(_) => eir::ir::Variant::new(
                            self.type_compiler.compile(coercion.from())?,
                            argument,
                        )
                        .into(),
                        Type::List(list_type) => eir::ir::Variant::new(
                            self.type_compiler.compile_list(list_type)?,
                            eir::ir::Record::new(
                                self.type_compiler.compile_list(list_type)?,
                                vec![argument],
                            ),
                        )
                        .into(),
                        Type::Any(_) | Type::Union(_) => argument,
                        Type::Reference(_) | Type::Unknown(_) | Type::Variable(_) => {
                            unreachable!()
                        }
                    }
                }
            }
            Expression::Variable(variable) => self
                .expression_compiler_set
                .variable_compiler
                .compile(variable)?,
            Expression::RecordUpdate(_) => unreachable!(),
        })
    }

    fn compile_let(&self, let_: &Let) -> Result<eir::ir::Expression, CompileError> {
        let_.definitions().iter().rev().fold(
            self.compile(let_.expression()),
            |expression, definition| {
                Ok(match definition {
                    Definition::FunctionDefinition(definition) => {
                        let type_ = self
                            .reference_type_resolver
                            .resolve_to_function(definition.type_())?
                            .unwrap();

                        eir::ir::LetRecursive::new(
                            eir::ir::Definition::new(
                                definition.name(),
                                definition
                                    .arguments()
                                    .iter()
                                    .zip(type_.arguments())
                                    .map(|(name, type_)| {
                                        Ok(eir::ir::Argument::new(
                                            name.clone(),
                                            self.type_compiler.compile(type_)?,
                                        ))
                                    })
                                    .collect::<Result<_, CompileError>>()?,
                                self.compile(definition.body())?,
                                self.type_compiler.compile(
                                    &self.last_result_type_calculator.calculate(
                                        definition.type_(),
                                        definition.arguments().len(),
                                    )?,
                                )?,
                            ),
                            expression?,
                        )
                        .into()
                    }
                    Definition::VariableDefinition(definition) => eir::ir::Let::new(
                        definition.name(),
                        self.type_compiler.compile(definition.type_())?,
                        self.compile(definition.body())?,
                        expression?,
                    )
                    .into(),
                })
            },
        )
    }

    fn compile_let_error(&self, let_: &LetError) -> Result<eir::ir::Expression, CompileError> {
        self.compile(
            &self
                .expression_transformer_set
                .let_error_transformer
                .transform(let_)?,
        )
    }

    fn compile_case(&self, case: &Case) -> Result<eir::ir::Expression, CompileError> {
        if !self.reference_type_resolver.is_any(case.type_())?
            && !self.reference_type_resolver.is_union(case.type_())?
        {
            return Err(CompileError::CaseArgumentTypeInvalid(
                case.source_information().clone(),
            ));
        }

        Ok(eir::ir::Let::new(
            case.name(),
            self.type_compiler.compile(case.type_())?,
            self.compile(case.argument())?,
            eir::ir::Case::new(
                eir::ir::Variable::new(case.name()),
                case.alternatives()
                    .iter()
                    .map(|alternative| self.compile_alternative(alternative, case.name()))
                    .collect::<Result<Vec<Option<Vec<_>>>, CompileError>>()?
                    .into_iter()
                    .fuse()
                    .flatten()
                    .flatten()
                    .collect(),
                case.alternatives()
                    .iter()
                    .map(|alternative| -> Result<_, CompileError> {
                        Ok(
                            if self.reference_type_resolver.is_any(alternative.type_())? {
                                Some(eir::ir::DefaultAlternative::new(
                                    case.name(),
                                    self.compile(alternative.expression())?,
                                ))
                            } else {
                                None
                            },
                        )
                    })
                    .collect::<Result<Vec<Option<_>>, _>>()?
                    .into_iter()
                    .flatten()
                    .next(),
            ),
        )
        .into())
    }

    fn compile_alternative(
        &self,
        alternative: &Alternative,
        variable_name: &str,
    ) -> Result<Option<Vec<eir::ir::Alternative>>, CompileError> {
        Ok(
            match &self.reference_type_resolver.resolve(alternative.type_())? {
                Type::Any(_) => None,
                Type::Boolean(_)
                | Type::Function(_)
                | Type::None(_)
                | Type::Number(_)
                | Type::Record(_)
                | Type::String(_) => Some(vec![eir::ir::Alternative::new(
                    self.type_compiler.compile(alternative.type_())?,
                    variable_name,
                    self.compile(alternative.expression())?,
                )]),
                Type::List(list_type) => {
                    let list_type = self.type_compiler.compile_list(list_type)?;

                    Some(vec![eir::ir::Alternative::new(
                        list_type.clone(),
                        variable_name,
                        eir::ir::Let::new(
                            variable_name,
                            self.type_compiler.compile_any_list(),
                            eir::ir::RecordElement::new(
                                list_type,
                                0,
                                eir::ir::Variable::new(variable_name),
                            ),
                            self.compile(alternative.expression())?,
                        ),
                    )])
                }
                Type::Union(union_type) => Some(
                    union_type
                        .types()
                        .iter()
                        .map(|type_| -> Result<_, CompileError> {
                            let type_ = self.type_compiler.compile(type_)?;

                            Ok(eir::ir::Alternative::new(
                                type_.clone(),
                                variable_name,
                                eir::ir::Let::new(
                                    variable_name,
                                    self.type_compiler.compile_union(),
                                    eir::ir::Variant::new(
                                        type_,
                                        eir::ir::Variable::new(variable_name),
                                    ),
                                    self.compile(alternative.expression())?,
                                ),
                            ))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                Type::Reference(_) | Type::Unknown(_) | Type::Variable(_) => {
                    unreachable!()
                }
            },
        )
    }

    fn compile_arithmetic_operator(operator: ArithmeticOperator) -> eir::ir::ArithmeticOperator {
        match operator {
            ArithmeticOperator::Add => eir::ir::ArithmeticOperator::Add,
            ArithmeticOperator::Subtract => eir::ir::ArithmeticOperator::Subtract,
            ArithmeticOperator::Multiply => eir::ir::ArithmeticOperator::Multiply,
            ArithmeticOperator::Divide => eir::ir::ArithmeticOperator::Divide,
        }
    }

    fn compile_order_operator(operator: OrderOperator) -> eir::ir::ComparisonOperator {
        match operator {
            OrderOperator::LessThan => eir::ir::ComparisonOperator::LessThan,
            OrderOperator::LessThanOrEqual => eir::ir::ComparisonOperator::LessThanOrEqual,
            OrderOperator::GreaterThan => eir::ir::ComparisonOperator::GreaterThan,
            OrderOperator::GreaterThanOrEqual => eir::ir::ComparisonOperator::GreaterThanOrEqual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
            type_canonicalizer::TypeCanonicalizer,
            type_comparability_checker::TypeComparabilityChecker,
            type_equality_checker::TypeEqualityChecker, type_id_calculator::TypeIdCalculator,
        },
        *,
    };
    use crate::{debug::SourceInformation, types};
    use pretty_assertions::assert_eq;

    fn create_expression_compiler(module: &Module) -> (Arc<ExpressionCompiler>, Arc<TypeCompiler>) {
        let reference_type_resolver = ReferenceTypeResolver::new(module);
        let last_result_type_calculator =
            LastResultTypeCalculator::new(reference_type_resolver.clone());
        let type_id_calculator = TypeIdCalculator::new(reference_type_resolver.clone());
        let type_compiler = TypeCompiler::new(
            reference_type_resolver.clone(),
            type_id_calculator,
            LIST_TYPE_CONFIGURATION.clone(),
        );
        let variable_compiler = VariableCompiler::new(
            type_compiler.clone(),
            reference_type_resolver.clone(),
            module,
        )
        .unwrap();
        let type_comparability_checker =
            TypeComparabilityChecker::new(reference_type_resolver.clone());
        let type_equality_checker = TypeEqualityChecker::new(reference_type_resolver.clone());
        let type_canonicalizer = TypeCanonicalizer::new(
            reference_type_resolver.clone(),
            type_equality_checker.clone(),
        );
        let equal_operation_transformer = EqualOperationTransformer::new(
            reference_type_resolver.clone(),
            type_comparability_checker,
            type_equality_checker.clone(),
            LIST_TYPE_CONFIGURATION.clone(),
        );
        let not_equal_operation_transformer = NotEqualOperationTransformer::new();
        let list_literal_transformer = ListLiteralTransformer::new(
            reference_type_resolver.clone(),
            LIST_TYPE_CONFIGURATION.clone(),
        );
        let boolean_operation_transformer = BooleanOperationTransformer::new();
        let function_type_coercion_transformer = FunctionTypeCoercionTransformer::new(
            type_equality_checker.clone(),
            reference_type_resolver.clone(),
        );
        let list_type_coercion_transformer = ListTypeCoercionTransformer::new(
            type_equality_checker.clone(),
            reference_type_resolver.clone(),
            LIST_TYPE_CONFIGURATION.clone(),
        );
        let list_case_transformer = ListCaseTransformer::new(
            reference_type_resolver.clone(),
            LIST_TYPE_CONFIGURATION.clone(),
        );
        let let_error_transformer = LetErrorTransformer::new(
            reference_type_resolver.clone(),
            type_equality_checker,
            type_canonicalizer,
            ERROR_TYPE_CONFIGURATION.clone(),
        );

        (
            ExpressionCompiler::new(
                ExpressionCompilerSet { variable_compiler }.into(),
                ExpressionTransformerSet {
                    equal_operation_transformer,
                    not_equal_operation_transformer,
                    list_literal_transformer,
                    boolean_operation_transformer,
                    function_type_coercion_transformer,
                    list_type_coercion_transformer,
                    list_case_transformer,
                    let_error_transformer,
                }
                .into(),
                reference_type_resolver,
                last_result_type_calculator,
                type_compiler.clone(),
                STRING_TYPE_CONFIGURATION.clone(),
            ),
            type_compiler,
        )
    }

    mod operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_arithmetic_operation() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            assert_eq!(
                expression_compiler.compile(
                    &ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(2.0, SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into(),
                ),
                Ok(
                    eir::ir::ArithmeticOperation::new(eir::ir::ArithmeticOperator::Add, 1.0, 2.0)
                        .into()
                )
            );
        }

        #[test]
        fn compile_number_comparison_operation() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            assert_eq!(
                expression_compiler.compile(
                    &OrderOperation::new(
                        OrderOperator::LessThan,
                        Number::new(1.0, SourceInformation::dummy()),
                        Number::new(2.0, SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into(),
                ),
                Ok(eir::ir::ComparisonOperation::new(
                    eir::ir::ComparisonOperator::LessThan,
                    1.0,
                    2.0
                )
                .into())
            );
        }

        #[test]
        fn compile_pipe_operation() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            assert_eq!(
                expression_compiler.compile(
                    &PipeOperation::with_type(
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        ),
                        Number::new(1.0, SourceInformation::dummy()),
                        Variable::new("f", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into(),
                ),
                Ok(eir::ir::FunctionApplication::new(
                    eir::types::Function::new(eir::types::Type::Number, eir::types::Type::Number),
                    eir::ir::Variable::new("f"),
                    1.0
                )
                .into())
            );
        }
    }

    #[test]
    fn compile_let() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &Let::new(
                    vec![VariableDefinition::new(
                        "x",
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::Let::new(
                "x",
                eir::types::Type::Number,
                42.0,
                eir::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_with_multiple_definitions() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &Let::new(
                    vec![
                        VariableDefinition::new(
                            "x",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into(),
                        VariableDefinition::new(
                            "y",
                            Number::new(42.0, SourceInformation::dummy()),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy()
                        )
                        .into()
                    ],
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::Let::new(
                "x",
                eir::types::Type::Number,
                42.0,
                eir::ir::Let::new(
                    "y",
                    eir::types::Type::Number,
                    42.0,
                    eir::ir::Variable::new("x")
                )
            )
            .into())
        );
    }

    #[test]
    fn compile_let_with_function_definition() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
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
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::LetRecursive::new(
                eir::ir::Definition::new(
                    "f",
                    vec![eir::ir::Argument::new("x", eir::types::Type::Number)],
                    42.0,
                    eir::types::Type::Number,
                ),
                eir::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_with_recursive_functions() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &Let::new(
                    vec![FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Application::with_type(
                            types::Function::new(
                                types::Number::new(SourceInformation::dummy()),
                                types::Number::new(SourceInformation::dummy()),
                                SourceInformation::dummy(),
                            ),
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
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::LetRecursive::new(
                eir::ir::Definition::new(
                    "f",
                    vec![eir::ir::Argument::new("x", eir::types::Type::Number)],
                    eir::ir::FunctionApplication::new(
                        eir::types::Function::new(
                            eir::types::Type::Number,
                            eir::types::Type::Number
                        ),
                        eir::ir::Variable::new("f"),
                        eir::ir::Variable::new("x"),
                    ),
                    eir::types::Type::Number,
                ),
                eir::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_nested_let() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
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
                    Variable::new("x", SourceInformation::dummy()),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::LetRecursive::new(
                eir::ir::Definition::new(
                    "f",
                    vec![eir::ir::Argument::new("x", eir::types::Type::Number)],
                    eir::ir::LetRecursive::new(
                        eir::ir::Definition::new(
                            "g",
                            vec![eir::ir::Argument::new("y", eir::types::Type::Number)],
                            eir::ir::Variable::new("x"),
                            eir::types::Type::Number,
                        ),
                        eir::ir::Variable::new("x")
                    ),
                    eir::types::Type::Number,
                ),
                eir::ir::Variable::new("x")
            )
            .into())
        );
    }

    #[test]
    fn compile_let_with_free_variables() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &Let::new(
                    vec![VariableDefinition::new(
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
                        Variable::new("y", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    ),
                    SourceInformation::dummy()
                )
                .into(),
            ),
            Ok(eir::ir::Let::new(
                "y",
                eir::types::Type::Number,
                42.0,
                eir::ir::LetRecursive::new(
                    eir::ir::Definition::new(
                        "f",
                        vec![eir::ir::Argument::new("x", eir::types::Type::Number)],
                        eir::ir::Variable::new("y"),
                        eir::types::Type::Number,
                    ),
                    eir::ir::Variable::new("y")
                )
            )
            .into())
        );
    }

    #[test]
    fn compile_if_expressions() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &If::new(
                    Boolean::new(true, SourceInformation::dummy()),
                    Number::new(1.0, SourceInformation::dummy()),
                    Number::new(2.0, SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
                .into(),
            ),
            Ok(eir::ir::If::new(true, 1.0, 2.0).into())
        );
    }

    #[test]
    fn compile_case_expression_with_any_type_argument() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        insta::assert_debug_snapshot!(expression_compiler.compile(
            &Case::with_type(
                types::Any::new(SourceInformation::dummy()),
                "x",
                Variable::new("y", SourceInformation::dummy()),
                vec![
                    Alternative::new(
                        types::None::new(SourceInformation::dummy()),
                        None::new(SourceInformation::dummy())
                    ),
                    Alternative::new(
                        types::Any::new(SourceInformation::dummy()),
                        None::new(SourceInformation::dummy())
                    )
                ],
                SourceInformation::dummy(),
            )
            .into(),
        ));
    }

    #[test]
    fn fail_to_compile_case_expression_with_argument_type_invalid() {
        let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

        assert_eq!(
            expression_compiler.compile(
                &Case::with_type(
                    types::Boolean::new(SourceInformation::dummy()),
                    "x",
                    Boolean::new(true, SourceInformation::dummy()),
                    vec![Alternative::new(
                        types::Boolean::new(SourceInformation::dummy()),
                        Variable::new("x", SourceInformation::dummy())
                    )],
                    SourceInformation::dummy(),
                )
                .into(),
            ),
            Err(CompileError::CaseArgumentTypeInvalid(
                SourceInformation::dummy().into()
            ))
        );
    }

    #[test]
    fn compile_records() {
        let type_ = types::Record::new(
            "Foo",
            vec![types::RecordElement::new(
                "foo",
                types::Number::new(SourceInformation::dummy()),
            )],
            SourceInformation::dummy(),
        );
        let (expression_compiler, _) =
            create_expression_compiler(&Module::from_definitions_and_type_definitions(
                vec![TypeDefinition::new("Foo", type_)],
                vec![],
            ));

        assert_eq!(
            expression_compiler.compile(
                &RecordConstruction::new(
                    types::Reference::new("Foo", SourceInformation::dummy()),
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
            Ok(eir::ir::Record::new(eir::types::Record::new("Foo"), vec![42.0.into()]).into())
        );
    }

    mod type_coercion {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile_type_coercion_of_boolean() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            let union_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Boolean::new(true, SourceInformation::dummy()),
                        types::Boolean::new(SourceInformation::dummy()),
                        union_type,
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(eir::ir::Variant::new(eir::types::Type::Boolean, true).into())
            );
        }

        #[test]
        fn compile_type_coercion_of_record() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            let record_type =
                types::Record::new("Foo", Default::default(), SourceInformation::dummy());
            let union_type = types::Union::new(
                vec![
                    record_type.clone().into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        record_type,
                        union_type,
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(eir::ir::Variant::new(
                    eir::types::Record::new("Foo"),
                    eir::ir::Variable::new("x")
                )
                .into())
            );
        }

        #[test]
        fn compile_type_coercion_of_union() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            let lower_union_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );
            let upper_union_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        lower_union_type,
                        upper_union_type,
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(eir::ir::Variable::new("x").into())
            );
        }

        #[test]
        fn compile_type_coercion_from_any_type_to_any_type() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        types::Any::new(SourceInformation::dummy()),
                        types::Any::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(eir::ir::Variable::new("x").into())
            );
        }

        #[test]
        fn compile_type_coercion_from_non_union_type_to_any_type() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        types::Boolean::new(SourceInformation::dummy()),
                        types::Any::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(
                    eir::ir::Variant::new(eir::types::Type::Boolean, eir::ir::Variable::new("x"))
                        .into()
                )
            );
        }

        #[test]
        fn compile_type_coercion_from_union_type_to_any_type() {
            let (expression_compiler, _) = create_expression_compiler(&Module::dummy());

            let union_type = types::Union::new(
                vec![types::Boolean::new(SourceInformation::dummy()).into()],
                SourceInformation::dummy(),
            );

            assert_eq!(
                expression_compiler.compile(
                    &TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        union_type,
                        types::Any::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ),
                Ok(eir::ir::Variable::new("x").into())
            );
        }
    }
}
