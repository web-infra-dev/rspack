use rspack_core::{
  ConnectionState, DependencyConditionFn, DependencyId, EvaluatedInlinableValue, ExportsInfoGetter,
  GetUsedNameParam, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  PrefetchExportsInfoMode, RuntimeSpec, UsedName,
};
use rspack_util::ryu_js;
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, Program, VarDeclarator};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ESMImportSpecifierDependency,
  utils::eval::{
    BasicEvaluatedExpression, evaluate_to_boolean, evaluate_to_null, evaluate_to_number,
    evaluate_to_string, evaluate_to_undefined,
  },
  visitors::{JavascriptParser, TagInfoData, VariableDeclaration, VariableDeclarationKind},
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

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if !parser.has_inlinable_const_decls || for_name != INLINABLE_CONST_TAG {
      return None;
    }
    // Propagate inlinable constants. Help the rest const variable declarations that referencing the
    // inlinable constants to evaluate to an inlinable constants.
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = InlinableConstData::downcast(tag_info.data.clone()?);
    Some(match data.value {
      EvaluatedInlinableValue::Null => evaluate_to_null(start, end),
      EvaluatedInlinableValue::Undefined => evaluate_to_undefined(start, end),
      EvaluatedInlinableValue::Boolean(v) => evaluate_to_boolean(v, start, end),
      EvaluatedInlinableValue::Number(v) => evaluate_to_number(v, start, end),
      EvaluatedInlinableValue::String(v) => evaluate_to_string(v.to_string(), start, end),
    })
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !parser.has_inlinable_const_decls || !parser.is_top_level_scope() {
      return None;
    }
    if matches!(declaration.kind(), VariableDeclarationKind::Const)
      && let Some(name) = declarator.name.as_ident()
      && let Some(init) = &declarator.init
    {
      let evaluated = parser.evaluate_expression(init);
      if let Some(inlinable) = to_evaluated_inlinable_value(&evaluated) {
        parser.tag_variable(
          name.id.sym.clone(),
          INLINABLE_CONST_TAG,
          Some(InlinableConstData { value: inlinable }),
        );
      }
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
    && ryu_js::Buffer::new().format(num).len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_number(num))
  } else if evaluated.is_string()
    && let str = evaluated.string()
    && str.len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_string(str.as_str().into()))
  } else if evaluated.is_null() {
    Some(EvaluatedInlinableValue::new_null())
  } else if evaluated.is_undefined() {
    Some(EvaluatedInlinableValue::new_undefined())
  } else {
    None
  }
}

#[derive(Clone)]
pub struct InlineValueDependencyCondition {
  dependency_id: DependencyId,
}

impl InlineValueDependencyCondition {
  pub fn new(dependency_id: DependencyId) -> Self {
    Self { dependency_id }
  }
}

impl DependencyConditionFn for InlineValueDependencyCondition {
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
    let inline_const_enabled = module
      .as_normal_module()
      .and_then(|m| m.get_parser_options())
      .and_then(|options| options.get_javascript())
      .map(|options| options.inline_const == Some(true))
      .unwrap_or_default();
    let inline_enum_enabled = module
      .build_info()
      .collected_typescript_info
      .as_ref()
      .map(|info| !info.exported_enums.is_empty())
      .unwrap_or_default();
    if !inline_const_enabled && !inline_enum_enabled {
      // bailout if the target module didn't enable inline const/enum
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
    if ids.is_empty() {
      return bailout;
    }
    let exports_info = mg.get_prefetched_exports_info(module, PrefetchExportsInfoMode::Nested(ids));
    if matches!(
      ExportsInfoGetter::get_used_name(GetUsedNameParam::WithNames(&exports_info), runtime, ids),
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
