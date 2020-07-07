use super::expression::Expression;
use crate::debug::SourceInformation;
use crate::types::Type;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordConstruction {
    type_: Type,
    elements: BTreeMap<String, Expression>,
    source_information: Arc<SourceInformation>,
}

impl RecordConstruction {
    pub fn new(
        type_: impl Into<Type>,
        elements: BTreeMap<String, Expression>,
        source_information: impl Into<Arc<SourceInformation>>,
    ) -> Self {
        Self {
            type_: type_.into(),
            elements,
            source_information: source_information.into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn elements(&self) -> &BTreeMap<String, Expression> {
        &self.elements
    }

    pub fn source_information(&self) -> &Arc<SourceInformation> {
        &self.source_information
    }

    pub fn convert_expressions<E>(
        &self,
        convert: &mut impl FnMut(&Expression) -> Result<Expression, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.type_.clone(),
            self.elements
                .iter()
                .map(|(name, expression)| {
                    Ok((name.into(), expression.convert_expressions(convert)?))
                })
                .collect::<Result<_, _>>()?,
            self.source_information.clone(),
        ))
    }

    pub fn convert_types<E>(
        &self,
        convert: &mut impl FnMut(&Type) -> Result<Type, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.type_.convert_types(convert)?,
            self.elements
                .iter()
                .map(|(name, expression)| Ok((name.into(), expression.convert_types(convert)?)))
                .collect::<Result<_, _>>()?,
            self.source_information.clone(),
        ))
    }
}
