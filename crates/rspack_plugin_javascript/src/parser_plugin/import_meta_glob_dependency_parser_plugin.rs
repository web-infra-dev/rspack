use rspack_core::{
  ContextMode, ContextModulePattern, ContextOptions, DependencyCategory, extract_glob_base_dir,
  get_context, glob_base_dir_end,
};
use rspack_util::SpanExt;
use swc_core::{common::Spanned, ecma::ast::CallExpr};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaGlobDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::get_bool_by_obj_prop,
  },
  visitors::{JavascriptParser, expr_name, static_string_from_expr},
};

fn create_import_meta_glob_dependency(
  node: &CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaGlobDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  let glob_pattern = static_string_from_expr(&dyn_imported.expr)?;

  let recursive = {
    let base_dir_end = glob_base_dir_end(&glob_pattern);
    glob_pattern.contains("**") || glob_pattern[base_dir_end..].contains('/')
  };
  let context = get_context(parser.resource_data);
  let context_glob_pattern = if let Some(pattern) = glob_pattern.strip_prefix('/') {
    format!("{}/{}", parser.compiler_options.context.as_str(), pattern)
  } else {
    format!("{}/{}", context.as_str(), glob_pattern)
  };
  let base_dir = extract_glob_base_dir(&context_glob_pattern).to_string();

  let mode = node
    .args
    .get(1)
    .and_then(|arg| arg.expr.as_object())
    .map_or(ContextMode::Lazy, |obj| {
      if get_bool_by_obj_prop(obj, "eager").is_some_and(|b| b.value) {
        ContextMode::Sync
      } else {
        ContextMode::Lazy
      }
    });

  let context_options = ContextOptions {
    pattern: ContextModulePattern::Glob(glob_pattern),
    recursive,
    category: DependencyCategory::Esm,
    request: base_dir.clone(),
    context: base_dir,
    mode,
    start: node.span().real_lo(),
    end: node.span().real_hi(),
    ..Default::default()
  };
  Some(ImportMetaGlobDependency::new(
    context_options,
    node.span.into(),
    parser.in_try,
  ))
}

pub struct ImportMetaGlobDependencyParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for ImportMetaGlobDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::IMPORT_META_GLOB {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_GLOB.into(),
        expr_name::IMPORT_META.into(),
        Some(true),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != expr_name::IMPORT_META_GLOB || expr.args.is_empty() || expr.args.len() > 2 {
      None
    } else if let Some(dep) = create_import_meta_glob_dependency(expr, parser) {
      parser.add_dependency(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
