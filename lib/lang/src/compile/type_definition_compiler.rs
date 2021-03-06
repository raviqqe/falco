use super::{
    error::CompileError,
    reference_type_resolver::ReferenceTypeResolver,
    type_compiler::{TypeCompiler, NONE_TYPE_NAME, THUNK_ARGUMENT_TYPE_NAME},
};
use crate::{ast::*, types::Type};
use std::{collections::HashSet, sync::Arc};

pub struct TypeDefinitionCompiler {
    type_compiler: Arc<TypeCompiler>,
    reference_type_resolver: Arc<ReferenceTypeResolver>,
}

impl TypeDefinitionCompiler {
    pub fn new(
        type_compiler: Arc<TypeCompiler>,
        reference_type_resolver: Arc<ReferenceTypeResolver>,
    ) -> Arc<Self> {
        Self {
            type_compiler,
            reference_type_resolver,
        }
        .into()
    }

    pub fn compile(&self, module: &Module) -> Result<Vec<eir::ir::TypeDefinition>, CompileError> {
        let mut definitions = vec![
            eir::ir::TypeDefinition::new(
                THUNK_ARGUMENT_TYPE_NAME,
                eir::types::RecordBody::new(vec![]),
            ),
            eir::ir::TypeDefinition::new(NONE_TYPE_NAME, eir::types::RecordBody::new(vec![])),
        ]
        .into_iter()
        .chain(
            self.collect_types(module)?
                .iter()
                .map(|type_| self.compile_type_definitions(type_))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        )
        .collect::<Vec<_>>();

        definitions.sort_by_key(|definition| definition.name().to_string());
        definitions.dedup_by_key(|definition| definition.name().to_string());

        Ok(definitions)
    }

    fn collect_types(&self, module: &Module) -> Result<HashSet<Type>, CompileError> {
        let mut types = module
            .imports()
            .iter()
            .flat_map(|import| import.module_interface().types().values().cloned())
            .collect::<HashSet<_>>();

        module.transform_types(&mut |type_| -> Result<Type, CompileError> {
            types.insert(type_.clone());

            Ok(type_.clone())
        })?;

        Ok(types)
    }

    fn compile_type_definitions(
        &self,
        type_: &Type,
    ) -> Result<Vec<eir::ir::TypeDefinition>, CompileError> {
        Ok(match &self.reference_type_resolver.resolve(type_)? {
            Type::List(list) => vec![eir::ir::TypeDefinition::new(
                self.type_compiler.compile_list(list)?.name(),
                eir::types::RecordBody::new(vec![self.type_compiler.compile_any_list().into()]),
            )],
            Type::Record(record) => vec![eir::ir::TypeDefinition::new(
                record.name(),
                self.type_compiler.compile_record_body(record)?,
            )],
            Type::Union(union) => union
                .types()
                .iter()
                .map(|type_| self.compile_type_definitions(type_))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),
            Type::Any(_)
            | Type::Boolean(_)
            | Type::Function(_)
            | Type::None(_)
            | Type::Number(_)
            | Type::String(_) => vec![],
            Type::Reference(_) | Type::Unknown(_) | Type::Variable(_) => unreachable!(),
        })
    }
}
