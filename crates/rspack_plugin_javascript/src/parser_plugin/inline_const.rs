use rspack_core::{
  ConnectionState, DependencyConditionFn, DependencyId, EvaluatedInlinableValue, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeSpec, UsedName,
};
use swc_core::ecma::{
  ast::{ModuleDecl, ModuleItem, Program, VarDeclKind},
  utils::number::ToJsString,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ESMImportSpecifierDependency,
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
          let evaluated = parser.evaluate_expression(init);
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
    Some(EvaluatedInlinableValue::new_boolean(evaluated.bool()))
  } else if evaluated.is_number()
    && let num = evaluated.number()
    && let num = num.to_js_string()
    && num.len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_short_number(&num))
  } else if evaluated.is_string()
    && let str = evaluated.string()
    && str.len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_short_string(str))
  } else if evaluated.is_null() {
    Some(EvaluatedInlinableValue::new_null())
  } else if evaluated.is_undefined() {
    Some(EvaluatedInlinableValue::new_undefined())
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
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let bailout = ConnectionState::Active(true);
    let module = mg
      .get_module_by_dependency_id(&self.dependency_id)
      .expect("should have target module");
    if let Some(module) = module.as_normal_module()
      && let Some(options) = module.get_parser_options()
      && let Some(options) = options.get_javascript()
      && options.inline_const != Some(true)
    {
      // bailout if the target module didn't enable inline const
      return bailout;
    }
    let module = &module.identifier();
    let dependency = mg
      .dependency_by_id(&self.dependency_id)
      .expect("should have dependency");
    let dependency = dependency
      .downcast_ref::<ESMImportSpecifierDependency>()
      .expect("should be ESMImportSpecifierDependency");
    let ids = dependency.get_ids(mg);
    let exports_info = mg.get_exports_info(module);
    if matches!(
      exports_info.get_used_name(mg, runtime, ids),
      Some(UsedName::Inlined(_))
    ) {
      return ConnectionState::Active(false);
    }
    bailout
  }

  fn handle_composed(&self, primary: ConnectionState, rest: ConnectionState) -> ConnectionState {
    if primary.is_false() {
      return primary;
    }
    rest
  }
}
