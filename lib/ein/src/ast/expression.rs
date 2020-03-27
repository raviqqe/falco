use super::application::Application;
use super::boolean::Boolean;
use super::case::Case;
use super::definition::Definition;
use super::if_::If;
use super::let_::Let;
use super::none::None;
use super::number::Number;
use super::operation::Operation;
use super::record::Record;
use super::variable::Variable;
use crate::types::Type;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Application(Application),
    Boolean(Boolean),
    Case(Case),
    Record(Record),
    If(If),
    Let(Let),
    None(None),
    Number(Number),
    Operation(Operation),
    Variable(Variable),
}

impl Expression {
    pub fn substitute_type_variables(&self, substitutions: &HashMap<usize, Type>) -> Self {
        match self {
            Self::Application(application) => {
                application.substitute_type_variables(substitutions).into()
            }
            Self::Case(case) => case.substitute_type_variables(substitutions).into(),
            Self::Record(record) => record.substitute_type_variables(substitutions).into(),
            Self::If(if_) => if_.substitute_type_variables(substitutions).into(),
            Self::Let(let_) => let_.substitute_type_variables(substitutions).into(),
            Self::Operation(operation) => operation.substitute_type_variables(substitutions).into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub fn convert_definitions(&self, convert: &mut impl FnMut(&Definition) -> Definition) -> Self {
        match self {
            Self::Application(application) => application.convert_definitions(convert).into(),
            Self::Case(case) => case.convert_definitions(convert).into(),
            Self::Record(record) => record.convert_definitions(convert).into(),
            Self::If(if_) => if_.convert_definitions(convert).into(),
            Self::Let(let_) => let_.convert_definitions(convert).into(),
            Self::Operation(operation) => operation.convert_definitions(convert).into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub fn convert_expressions(&self, convert: &mut impl FnMut(&Expression) -> Expression) -> Self {
        let expression = match self {
            Self::Application(application) => application.convert_expressions(convert).into(),
            Self::Case(case) => case.convert_expressions(convert).into(),
            Self::Record(record) => record.convert_expressions(convert).into(),
            Self::If(if_) => if_.convert_expressions(convert).into(),
            Self::Let(let_) => let_.convert_expressions(convert).into(),
            Self::Operation(operation) => operation.convert_expressions(convert).into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        };

        convert(&expression)
    }

    pub fn convert_types(&self, convert: &mut impl FnMut(&Type) -> Type) -> Self {
        match self {
            Self::Application(application) => application.convert_types(convert).into(),
            Self::Case(case) => case.convert_types(convert).into(),
            Self::Record(record) => record.convert_types(convert).into(),
            Self::If(if_) => if_.convert_types(convert).into(),
            Self::Let(let_) => let_.convert_types(convert).into(),
            Self::Operation(operation) => operation.convert_types(convert).into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub fn resolve_reference_types(&self, environment: &HashMap<String, Type>) -> Self {
        match self {
            Self::Application(application) => {
                application.resolve_reference_types(environment).into()
            }
            Self::Case(case) => case.resolve_reference_types(environment).into(),
            Self::Record(record) => record.resolve_reference_types(environment).into(),
            Self::If(if_) => if_.resolve_reference_types(environment).into(),
            Self::Let(let_) => let_.resolve_reference_types(environment).into(),
            Self::Operation(operation) => operation.resolve_reference_types(environment).into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        }
    }
}

impl From<Application> for Expression {
    fn from(application: Application) -> Expression {
        Self::Application(application)
    }
}

impl From<Boolean> for Expression {
    fn from(boolean: Boolean) -> Expression {
        Self::Boolean(boolean)
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Expression {
        Self::Case(case)
    }
}

impl From<Record> for Expression {
    fn from(record: Record) -> Expression {
        Self::Record(record)
    }
}

impl From<If> for Expression {
    fn from(if_: If) -> Expression {
        Self::If(if_)
    }
}

impl From<Let> for Expression {
    fn from(let_: Let) -> Expression {
        Self::Let(let_)
    }
}

impl From<None> for Expression {
    fn from(none: None) -> Expression {
        Self::None(none)
    }
}

impl From<Number> for Expression {
    fn from(number: Number) -> Expression {
        Self::Number(number)
    }
}

impl From<Operation> for Expression {
    fn from(operation: Operation) -> Expression {
        Self::Operation(operation)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Expression {
        Self::Variable(variable)
    }
}
