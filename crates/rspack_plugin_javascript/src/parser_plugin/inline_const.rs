use std::sync::Arc;

use rspack_core::{
  ConnectionState, DependencyCondition, DependencyConditionFn, DependencyId,
  EvaluatedInlinableValue, ModuleGraph, ModuleGraphConnection, RuntimeSpec, UsageState, UsedName,
};
use swc_core::ecma::{
  ast::{ExprStmt, ModuleDecl, ModuleItem, Program, VarDecl, VarDeclKind, VarDeclarator},
  utils::{number::ToJsString, ExprExt},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{self, ESMImportSpecifierDependency},
  utils::eval::BasicEvaluatedExpression,
  visitors::{JavascriptParser, Statement, TopLevelScope},
};

pub const INLINABLE_CONST_TAG: &str = "inlinable const";

#[derive(Debug, Clone)]
pub struct InlinableConstData {
  pub value: EvaluatedInlinableValue,
}

#[derive(Default)]
pub struct InlineConstPlugin;

impl JavascriptParserPlugin for InlineConstPlugin {
  fn program(&self, parser: &mut JavascriptParser, program: &Program) -> Option<bool> {
    if let Some(module) = program.as_module() {
      for item in &module.body {
        match item {
          ModuleItem::ModuleDecl(m) => {
            if m.is_import()
              || m.is_export_all()
              || matches!(m, ModuleDecl::ExportNamed(m) if m.src.is_some())
            {
              // For now we only handle cross-module const inlining, we might don't need to
              // inline const inside the module if we leave it to minimizer?
              parser.has_inlinable_const_decls = false;
              break;
            }
          }
          ModuleItem::Stmt(_) => continue,
        }
      }
    }

    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    if !parser.has_inlinable_const_decls || !matches!(parser.top_level_scope, TopLevelScope::Top) {
      return None;
    }

    if let Statement::Var(declaration) = stmt
      && matches!(declaration.kind, VarDeclKind::Const)
    {
      for declarator in &declaration.decls {
        if let Some(name) = declarator.name.as_ident()
          && let Some(init) = &declarator.init
        {
          let evaluated = parser.evaluate_expression(&init);
          if let Some(inlinable) = to_evaluated_inlinable_value(&evaluated) {
            parser.tag_variable(
              name.id.sym.to_string(),
              INLINABLE_CONST_TAG,
              Some(InlinableConstData { value: inlinable }),
            );
            continue;
          }
        }
      }
      return None;
    }

    None
  }
}

fn to_evaluated_inlinable_value(
  evaluated: &BasicEvaluatedExpression,
) -> Option<EvaluatedInlinableValue> {
  if evaluated.is_bool() {
    Some(EvaluatedInlinableValue::Boolean(evaluated.bool()))
  } else if evaluated.is_number()
    && let num = evaluated.number()
    && num.to_js_string().len() <= 8
  {
    Some(EvaluatedInlinableValue::ShortNumber(num))
  } else if evaluated.is_null() {
    Some(EvaluatedInlinableValue::Null)
  } else if evaluated.is_undefined() {
    Some(EvaluatedInlinableValue::Undefined)
  } else {
    None
  }
}

#[derive(Clone)]
pub struct InlineConstDependencyCondition {
  dependency_id: DependencyId,
}

impl InlineConstDependencyCondition {
  pub fn new(dependency_id: DependencyId) -> Self {
    Self { dependency_id }
  }
}

impl DependencyConditionFn for InlineConstDependencyCondition {
  fn get_connection_state(
    &self,
    _conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
  ) -> ConnectionState {
    let module = mg
      .module_identifier_by_dependency_id(&self.dependency_id)
      .expect("should have parent module");
    let dependency = mg
      .dependency_by_id(&self.dependency_id)
      .expect("should have dependency");
    let dependency = dependency
      .downcast_ref::<ESMImportSpecifierDependency>()
      .expect("should be ESMImportSpecifierDependency");
    let ids = dependency.get_ids(mg);
    let exports_info = mg.get_exports_info(module);
    // TODO: use get_used
    if matches!(
      exports_info.get_used_name(mg, runtime, ids),
      Some(UsedName::Inlined(_))
    ) {
      return ConnectionState::Bool(false);
    }
    ConnectionState::Bool(true)
  }

  fn handle_composed(&self, primary: ConnectionState, rest: ConnectionState) -> ConnectionState {
    if primary.is_false() {
      return primary;
    }
    rest
  }
}
