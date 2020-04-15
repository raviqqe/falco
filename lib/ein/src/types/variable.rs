use super::type_::Type;
use super::union::Union;
use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_VARIABLE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Variable {
    id: usize,
    inputs: Vec<Type>,
    source_information: Rc<SourceInformation>,
}

impl Variable {
    pub fn new(source_information: impl Into<Rc<SourceInformation>>) -> Self {
        Self::with_inputs(vec![], source_information)
    }

    pub fn with_inputs(
        inputs: Vec<Type>,
        source_information: impl Into<Rc<SourceInformation>>,
    ) -> Self {
        Self {
            id: GLOBAL_VARIABLE_ID.fetch_add(1, Ordering::SeqCst),
            inputs,
            source_information: source_information.into(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn inputs(&self) -> &[Type] {
        &self.inputs
    }

    pub fn source_information(&self) -> &Rc<SourceInformation> {
        &self.source_information
    }

    pub fn add_input(&self, type_: impl Into<Type>) -> Self {
        Self {
            id: self.id,
            inputs: self
                .inputs
                .iter()
                .cloned()
                .chain(vec![type_.into()])
                .collect(),
            source_information: self.source_information.clone(),
        }
    }

    pub fn simplify(&self) -> Type {
        Union::new(
            self.inputs
                .iter()
                .map(|type_| match type_ {
                    Type::Variable(variable) => variable.simplify(),
                    _ => type_.clone(),
                })
                .collect(),
            self.source_information.clone(),
        )
        .simplify()
    }

    pub fn substitute_variables(&self, substitutions: &HashMap<usize, Type>) -> Type {
        match substitutions.get(&self.id) {
            Some(type_) => type_.clone(),
            None => Self {
                id: self.id,
                inputs: self
                    .inputs
                    .iter()
                    .map(|type_| type_.substitute_variables(substitutions))
                    .collect(),
                source_information: self.source_information.clone(),
            }
            .into(),
        }
    }
}
