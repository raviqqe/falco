use super::application::Application;
use super::boolean::Boolean;
use super::case::Case;
use super::if_::If;
use super::let_::Let;
use super::list::List;
use super::none::None;
use super::number::Number;
use super::operation::Operation;
use super::record_construction::RecordConstruction;
use super::record_element_operation::RecordElementOperation;
use super::record_update::RecordUpdate;
use super::type_coercion::TypeCoercion;
use super::variable::Variable;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Application(Application),
    Boolean(Boolean),
    Case(Case),
    If(If),
    Let(Let),
    List(List),
    None(None),
    Number(Number),
    Operation(Operation),
    RecordConstruction(RecordConstruction),
    RecordElementOperation(RecordElementOperation),
    RecordUpdate(RecordUpdate),
    TypeCoercion(TypeCoercion),
    Variable(Variable),
}

impl Expression {
    pub fn convert_expressions<E>(
        &self,
        convert: &mut impl FnMut(&Expression) -> Result<Expression, E>,
    ) -> Result<Self, E> {
        let expression = match self {
            Self::Application(application) => application.convert_expressions(convert)?.into(),
            Self::Case(case) => case.convert_expressions(convert)?.into(),
            Self::RecordConstruction(record_construction) => {
                record_construction.convert_expressions(convert)?.into()
            }
            Self::RecordElementOperation(operation) => {
                operation.convert_expressions(convert)?.into()
            }
            Self::RecordUpdate(record_update) => record_update.convert_expressions(convert)?.into(),
            Self::If(if_) => if_.convert_expressions(convert)?.into(),
            Self::Let(let_) => let_.convert_expressions(convert)?.into(),
            Self::List(list) => list.convert_expressions(convert)?.into(),
            Self::Operation(operation) => operation.convert_expressions(convert)?.into(),
            Self::TypeCoercion(coercion) => coercion.convert_expressions(convert)?.into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        };

        convert(&expression)
    }

    pub fn convert_types<E>(
        &self,
        convert: &mut impl FnMut(&Type) -> Result<Type, E>,
    ) -> Result<Self, E> {
        Ok(match self {
            Self::Application(application) => application.convert_types(convert)?.into(),
            Self::Case(case) => case.convert_types(convert)?.into(),
            Self::RecordConstruction(record_construction) => {
                record_construction.convert_types(convert)?.into()
            }
            Self::RecordElementOperation(operation) => operation.convert_types(convert)?.into(),
            Self::RecordUpdate(record_update) => record_update.convert_types(convert)?.into(),
            Self::If(if_) => if_.convert_types(convert)?.into(),
            Self::Let(let_) => let_.convert_types(convert)?.into(),
            Self::List(list) => list.convert_types(convert)?.into(),
            Self::Operation(operation) => operation.convert_types(convert)?.into(),
            Self::TypeCoercion(coercion) => coercion.convert_types(convert)?.into(),
            Self::Boolean(_) | Self::None(_) | Self::Number(_) | Self::Variable(_) => self.clone(),
        })
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable(_))
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

impl From<RecordConstruction> for Expression {
    fn from(record_construction: RecordConstruction) -> Expression {
        Self::RecordConstruction(record_construction)
    }
}

impl From<RecordElementOperation> for Expression {
    fn from(operation: RecordElementOperation) -> Expression {
        Self::RecordElementOperation(operation)
    }
}

impl From<RecordUpdate> for Expression {
    fn from(record_update: RecordUpdate) -> Expression {
        Self::RecordUpdate(record_update)
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

impl From<List> for Expression {
    fn from(list: List) -> Expression {
        Self::List(list)
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

impl From<TypeCoercion> for Expression {
    fn from(coercion: TypeCoercion) -> Expression {
        Self::TypeCoercion(coercion)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Expression {
        Self::Variable(variable)
    }
}
