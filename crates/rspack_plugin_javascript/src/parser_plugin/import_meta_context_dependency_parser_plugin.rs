use rspack_core::{ContextMode, ContextNameSpaceObject, ContextOptions, DependencyCategory};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Lit},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ImportMetaContextDependency,
  utils::{
    eval::{self, BasicEvaluatedExpression},
    object_properties::{get_bool_by_obj_prop, get_literal_str_by_obj_prop, get_regex_by_obj_prop},
  },
  visitors::{JavascriptParser, clean_regexp_in_context_module, context_reg_exp, expr_name},
};

fn create_import_meta_context_dependency(
  node: &CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaContextDependency> {
  assert!(node.callee.is_expr());
  let dyn_imported = node.args.first()?;
  if dyn_imported.spread.is_some() {
    return None;
  }
  let context = dyn_imported
    .expr
    .as_lit()
    .and_then(|lit| {
      if let Lit::Str(str) = lit {
        return Some(str.value.to_string_lossy().to_string());
      }
      None
    })
    // TODO: should've used expression evaluation to handle cases like `abc${"efg"}`, etc.
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
  let reg = r"^\.\/.*$";
  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let regexp = get_regex_by_obj_prop(obj, "regExp");
    let regexp_span = regexp.map(|r| r.span().into());
    let regexp = regexp
      .map_or(RspackRegex::new(reg).expect("reg failed"), |regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let include = get_regex_by_obj_prop(obj, "include")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let exclude = get_regex_by_obj_prop(obj, "exclude")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let mode = get_literal_str_by_obj_prop(obj, "mode")
      .map_or(ContextMode::Sync, |s| s.value.to_string_lossy().as_ref().into());
    let recursive = get_bool_by_obj_prop(obj, "recursive")
      .is_none_or(|bool| bool.value);
    ContextOptions {
      reg_exp: clean_regexp_in_context_module(regexp, regexp_span, parser),
      include,
      exclude,
      recursive,
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      mode,
      replaces: Vec::new(),
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      referenced_exports: None,
      attributes: None,
    }
  } else {
    ContextOptions {
      recursive: true,
      mode: ContextMode::Sync,
      include: None,
      exclude: None,
      reg_exp: context_reg_exp(reg, "", None, parser),
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      replaces: Vec::new(),
      start: node.span().real_lo(),
      end: node.span().real_hi(),
      referenced_exports: None,
      attributes: None,
    }
  };
  Some(ImportMetaContextDependency::new(
    context_options,
    node.span.into(),
    parser.in_try,
  ))
}

pub struct ImportMetaContextDependencyParserPlugin;

impl JavascriptParserPlugin for ImportMetaContextDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::IMPORT_META_CONTEXT {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_CONTEXT.into(),
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
    if for_name != expr_name::IMPORT_META_CONTEXT || expr.args.is_empty() || expr.args.len() > 2 {
      None
    } else if let Some(dep) = create_import_meta_context_dependency(expr, parser) {
      parser.add_dependency(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
