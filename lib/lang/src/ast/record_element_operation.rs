use super::expression::Expression;
use crate::{debug::SourceInformation, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordElementOperation {
    type_: Type,
    key: String,
    argument: Arc<Expression>,
    source_information: Arc<SourceInformation>,
}

impl RecordElementOperation {
    pub fn new(
        type_: impl Into<Type>,
        key: impl Into<String>,
        argument: impl Into<Expression>,
        source_information: impl Into<Arc<SourceInformation>>,
    ) -> Self {
        Self {
            type_: type_.into(),
            key: key.into(),
            argument: Arc::new(argument.into()),
            source_information: source_information.into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn source_information(&self) -> &Arc<SourceInformation> {
        &self.source_information
    }

    pub fn transform_expressions<E>(
        &self,
        transform: &mut impl FnMut(&Expression) -> Result<Expression, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.type_.clone(),
            &self.key,
            self.argument.transform_expressions(transform)?,
            self.source_information.clone(),
        ))
    }

    pub fn transform_types<E>(
        &self,
        transform: &mut impl FnMut(&Type) -> Result<Type, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.type_.transform_types(transform)?,
            &self.key,
            self.argument.transform_types(transform)?,
            self.source_information.clone(),
        ))
    }
}
