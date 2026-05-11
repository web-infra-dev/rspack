use rspack_core::{
  ContextMode, ContextNameSpaceObject, ContextOptions, DependencyCategory, extract_glob_base_dir,
};
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Lit},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaGlobDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::get_bool_by_obj_prop,
  },
  visitors::{JavascriptParser, expr_name},
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
  let glob_pattern = dyn_imported
    .expr
    .as_lit()
    .and_then(|lit| {
      if let Lit::Str(str) = lit {
        return Some(str.value.to_string_lossy().to_string());
      }
      None
    })
    .or_else(|| {
      if let Some(tpl) = dyn_imported.expr.as_tpl()
        && tpl.exprs.is_empty()
        && tpl.quasis.len() == 1
        && let Some(el) = tpl.quasis.first()
      {
        return Some(el.raw.to_string());
      }
      None
    })?;

  let base_dir = extract_glob_base_dir(&glob_pattern).to_string();

  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let eager = get_bool_by_obj_prop(obj, "eager").is_some_and(|b| b.value);
    let mode = if eager {
      ContextMode::Sync
    } else {
      ContextMode::Lazy
    };
    let context = base_dir;
    ContextOptions {
      reg_exp: None,
      include: None,
      exclude: None,
      recursive: glob_pattern.contains("**"),
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      mode,
      replaces: Vec::new(),
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      referenced_specifiers: None,
      attributes: None,
      phase: None,
      glob_pattern: Some(glob_pattern),
    }
  } else {
    ContextOptions {
      reg_exp: None,
      include: None,
      exclude: None,
      recursive: glob_pattern.contains("**"),
      category: DependencyCategory::Esm,
      request: base_dir.clone(),
      context: base_dir,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      mode: ContextMode::Lazy,
      replaces: Vec::new(),
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      referenced_specifiers: None,
      attributes: None,
      phase: None,
      glob_pattern: Some(glob_pattern),
    }
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
