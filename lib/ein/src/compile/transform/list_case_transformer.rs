use super::super::error::CompileError;
use super::super::list_type_configuration::ListTypeConfiguration;
use super::super::reference_type_resolver::ReferenceTypeResolver;
use crate::ast::*;
use crate::types;
use std::sync::Arc;

pub struct ListCaseTransformer {
    reference_type_resolver: Arc<ReferenceTypeResolver>,
    configuration: Arc<ListTypeConfiguration>,
}

impl ListCaseTransformer {
    pub fn new(
        reference_type_resolver: Arc<ReferenceTypeResolver>,
        configuration: Arc<ListTypeConfiguration>,
    ) -> Arc<Self> {
        Self {
            reference_type_resolver,
            configuration,
        }
        .into()
    }

    pub fn transform(&self, case: &ListCase) -> Result<Expression, CompileError> {
        let source_information = case.source_information();
        let first_rest_type = types::Reference::new(
            self.configuration.first_rest_type_name(),
            source_information.clone(),
        );
        let none_type = types::None::new(source_information.clone());
        let element_type = self
            .reference_type_resolver
            .resolve_to_list(case.type_())?
            .unwrap()
            .element()
            .clone();

        Ok(Case::with_type(
            types::Union::new(
                vec![first_rest_type.clone().into(), none_type.clone().into()],
                source_information.clone(),
            ),
            "$firstRest",
            Application::new(
                Variable::new(
                    self.configuration.deconstruct_function_name(),
                    source_information.clone(),
                ),
                case.argument().clone(),
                source_information.clone(),
            ),
            vec![
                Alternative::new(none_type.clone(), case.empty_alternative().clone()),
                Alternative::new(
                    first_rest_type.clone(),
                    Let::new(
                        vec![
                            VariableDefinition::new(
                                case.first_name(),
                                Case::with_type(
                                    types::Any::new(source_information.clone()),
                                    "$element",
                                    Application::new(
                                        Variable::new(
                                            self.configuration.first_function_name(),
                                            source_information.clone(),
                                        ),
                                        Variable::new("$firstRest", source_information.clone()),
                                        source_information.clone(),
                                    ),
                                    vec![Alternative::new(
                                        element_type.clone(),
                                        Variable::new("$element", source_information.clone()),
                                    )],
                                    source_information.clone(),
                                ),
                                element_type.clone(),
                                source_information.clone(),
                            )
                            .into(),
                            VariableDefinition::new(
                                case.rest_name(),
                                Application::new(
                                    Variable::new(
                                        self.configuration.rest_function_name(),
                                        source_information.clone(),
                                    ),
                                    Variable::new("$firstRest", source_information.clone()),
                                    source_information.clone(),
                                ),
                                case.type_().clone(),
                                source_information.clone(),
                            )
                            .into(),
                        ],
                        case.non_empty_alternative().clone(),
                        source_information.clone(),
                    ),
                ),
            ],
            source_information.clone(),
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::list_type_configuration::LIST_TYPE_CONFIGURATION;
    use super::*;
    use crate::debug::*;

    fn create_list_case_transformer() -> Arc<ListCaseTransformer> {
        ListCaseTransformer::new(
            ReferenceTypeResolver::new(&Module::dummy()),
            LIST_TYPE_CONFIGURATION.clone(),
        )
    }

    #[test]
    fn transform() {
        insta::assert_debug_snapshot!(create_list_case_transformer().transform(&ListCase::new(
            Variable::new("xs", SourceInformation::dummy()),
            types::List::new(
                types::Number::new(SourceInformation::dummy()),
                SourceInformation::dummy(),
            ),
            "y",
            "ys",
            None::new(SourceInformation::dummy()),
            None::new(SourceInformation::dummy()),
            SourceInformation::dummy(),
        )));
    }
}
