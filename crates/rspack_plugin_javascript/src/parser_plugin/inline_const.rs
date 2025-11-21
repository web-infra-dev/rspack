use rspack_core::{
  Dependency, DependencyId, EvaluatedInlinableValue, ExportMode, ExportsInfoGetter,
  GetUsedNameParam, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, PrefetchExportsInfoMode,
  RuntimeSpec, UsageState, UsedName,
};
use rspack_util::{atom::Atom, ryu_js};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, Program, VarDeclarator};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{ESMExportImportedSpecifierDependency, ESMImportSpecifierDependency},
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

fn inline_value_enabled(dependency_id: &DependencyId, mg: &ModuleGraph) -> bool {
  let module = mg
    .get_module_by_dependency_id(dependency_id)
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
  inline_const_enabled || inline_enum_enabled
}

pub fn is_export_inlined(
  mg: &ModuleGraph,
  module: &ModuleIdentifier,
  ids: &[Atom],
  runtime: Option<&RuntimeSpec>,
) -> bool {
  let used_name = if ids.is_empty() {
    let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
      &mg.get_exports_info(module),
      mg,
      runtime,
      false,
    );
    ExportsInfoGetter::get_used_name(GetUsedNameParam::WithoutNames(&exports_info), runtime, ids)
  } else {
    let exports_info = mg.get_prefetched_exports_info(module, PrefetchExportsInfoMode::Nested(ids));
    ExportsInfoGetter::get_used_name(GetUsedNameParam::WithNames(&exports_info), runtime, ids)
  };
  matches!(used_name, Some(UsedName::Inlined(_)))
}

pub fn connection_active_inline_value_for_esm_import_specifier(
  dependency: &ESMImportSpecifierDependency,
  connection: &ModuleGraphConnection,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> bool {
  if !inline_value_enabled(dependency.id(), mg) {
    return true;
  }
  let module = connection.module_identifier();
  let ids = dependency.get_ids(mg);
  !is_export_inlined(mg, module, ids, runtime)
}

pub fn connection_active_inline_value_for_esm_export_imported_specifier(
  dependency: &ESMExportImportedSpecifierDependency,
  mode: &ExportMode,
  connection: &ModuleGraphConnection,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> bool {
  if !inline_value_enabled(dependency.id(), mg) {
    return true;
  }
  let ExportMode::NormalReexport(mode) = mode else {
    return true;
  };
  let module = connection.module_identifier();
  let exports_info = mg.get_exports_info(module).as_data(mg);
  if exports_info.other_exports_info().get_used(runtime) != UsageState::Unused {
    return true;
  }
  for item in &mode.items {
    if item.hidden || item.checked {
      return true;
    }
    if !is_export_inlined(mg, module, &item.ids, runtime) {
      return true;
    }
  }
  false
}
