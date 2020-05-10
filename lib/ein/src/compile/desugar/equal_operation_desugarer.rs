use super::super::error::CompileError;
use super::super::name_generator::NameGenerator;
use super::super::reference_type_resolver::ReferenceTypeResolver;
use super::super::type_equality_checker::TypeEqualityChecker;
use crate::ast::*;
use crate::debug::SourceInformation;
use crate::types::{self, Type};
use std::rc::Rc;

pub struct EqualOperationDesugarer {
    name_generator: NameGenerator,
    reference_type_resolver: Rc<ReferenceTypeResolver>,
    type_equality_checker: Rc<TypeEqualityChecker>,
}

impl EqualOperationDesugarer {
    pub fn new(
        reference_type_resolver: Rc<ReferenceTypeResolver>,
        type_equality_checker: Rc<TypeEqualityChecker>,
    ) -> Self {
        Self {
            name_generator: NameGenerator::new("equal_operation_argument_"),
            reference_type_resolver,
            type_equality_checker,
        }
    }

    pub fn desugar(&mut self, module: &Module) -> Result<Module, CompileError> {
        let module =
            module.convert_expressions(&mut |expression| -> Result<Expression, CompileError> {
                self.desugar_expression(expression)
            })?;

        Ok(Module::new(
            module.path().clone(),
            module.export().clone(),
            module.imported_modules().to_vec(),
            module.type_definitions().to_vec(),
            module
                .definitions()
                .iter()
                .cloned()
                .chain(
                    module
                        .type_definitions()
                        .iter()
                        .filter_map(|type_definition| {
                            if let Type::Record(record_type) = type_definition.type_() {
                                Some(
                                    self.create_record_equal_function(record_type)
                                        .map(Definition::from),
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .collect(),
        ))
    }

    fn create_record_equal_function(
        &mut self,
        record_type: &types::Record,
    ) -> Result<FunctionDefinition, CompileError> {
        let source_information = record_type.source_information();
        let mut expression: Expression = Boolean::new(true, source_information.clone()).into();

        for (key, element_type) in record_type.elements() {
            expression = If::new(
                expression,
                self.desugar_equal_operation(
                    element_type,
                    &RecordElementOperation::new(
                        record_type.clone(),
                        key,
                        Variable::new("lhs", source_information.clone()),
                        source_information.clone(),
                    )
                    .into(),
                    &RecordElementOperation::new(
                        record_type.clone(),
                        key,
                        Variable::new("rhs", source_information.clone()),
                        source_information.clone(),
                    )
                    .into(),
                    source_information.clone(),
                )?,
                Boolean::new(false, source_information.clone()),
                source_information.clone(),
            )
            .into();
        }

        Ok(FunctionDefinition::new(
            self.get_record_equal_function_name(record_type),
            vec!["lhs".into(), "rhs".into()],
            expression,
            types::Function::new(
                record_type.clone(),
                types::Function::new(
                    record_type.clone(),
                    types::Boolean::new(source_information.clone()),
                    source_information.clone(),
                ),
                source_information.clone(),
            ),
            source_information.clone(),
        ))
    }

    fn desugar_expression(&mut self, expression: &Expression) -> Result<Expression, CompileError> {
        Ok(if let Expression::Operation(operation) = expression {
            if operation.operator() == Operator::Equal {
                self.desugar_equal_operation(
                    operation.type_(),
                    operation.lhs(),
                    operation.rhs(),
                    operation.source_information().clone(),
                )?
            } else {
                expression.clone()
            }
        } else {
            expression.clone()
        })
    }

    fn desugar_equal_operation(
        &mut self,
        type_: &Type,
        lhs: &Expression,
        rhs: &Expression,
        source_information: Rc<SourceInformation>,
    ) -> Result<Expression, CompileError> {
        Ok(match self.reference_type_resolver.resolve(type_)? {
            Type::Boolean(_) => If::new(
                lhs.clone(),
                If::new(
                    rhs.clone(),
                    Boolean::new(true, source_information.clone()),
                    Boolean::new(false, source_information.clone()),
                    source_information.clone(),
                ),
                If::new(
                    rhs.clone(),
                    Boolean::new(false, source_information.clone()),
                    Boolean::new(true, source_information.clone()),
                    source_information.clone(),
                ),
                source_information,
            )
            .into(),
            Type::Function(_) => Boolean::new(false, source_information).into(),
            Type::None(_) => Boolean::new(true, source_information).into(),
            Type::Number(_) => Operation::with_type(
                type_.clone(),
                Operator::Equal,
                lhs.clone(),
                rhs.clone(),
                source_information,
            )
            .into(),
            Type::Record(record) => Application::new(
                Application::new(
                    Variable::new(
                        self.get_record_equal_function_name(&record),
                        source_information.clone(),
                    ),
                    lhs.clone(),
                    source_information.clone(),
                ),
                rhs.clone(),
                source_information,
            )
            .into(),
            Type::Union(union) => {
                let lhs_name = self.name_generator.generate();
                let rhs_name = self.name_generator.generate();

                Case::with_type(
                    union.clone(),
                    &lhs_name,
                    lhs.clone(),
                    union
                        .types()
                        .iter()
                        .map(|lhs_type| {
                            Ok(Alternative::new(
                                lhs_type.clone(),
                                Case::with_type(
                                    union.clone(),
                                    &rhs_name,
                                    rhs.clone(),
                                    union
                                        .types()
                                        .iter()
                                        .map(|rhs_type| {
                                            Ok(Alternative::new(
                                                rhs_type.clone(),
                                                if self
                                                    .type_equality_checker
                                                    .equal(lhs_type, rhs_type)?
                                                {
                                                    self.desugar_equal_operation(
                                                        rhs_type,
                                                        &Variable::new(
                                                            &lhs_name,
                                                            source_information.clone(),
                                                        )
                                                        .into(),
                                                        &Variable::new(
                                                            &rhs_name,
                                                            source_information.clone(),
                                                        )
                                                        .into(),
                                                        source_information.clone(),
                                                    )?
                                                } else {
                                                    Boolean::new(false, source_information.clone())
                                                        .into()
                                                },
                                            ))
                                        })
                                        .collect::<Result<_, CompileError>>()?,
                                    source_information.clone(),
                                ),
                            ))
                        })
                        .collect::<Result<_, CompileError>>()?,
                    source_information,
                )
                .into()
            }
            Type::Reference(_) | Type::Unknown(_) | Type::Variable(_) => unreachable!(),
        })
    }

    fn get_record_equal_function_name(&self, record_type: &types::Record) -> String {
        format!("{}.$equal", record_type.name())
    }
}
