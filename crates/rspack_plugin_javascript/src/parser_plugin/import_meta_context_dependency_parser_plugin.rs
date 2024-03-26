use rspack_core::{
  clean_regexp_in_context_module, context_reg_exp, ContextMode, ContextNameSpaceObject,
  ContextOptions, DependencyCategory, SpanExt,
};
use rspack_regex::{regexp_as_str, RspackRegex};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Lit};

use super::JavascriptParserPlugin;
use crate::dependency::ImportMetaContextDependency;
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::utils::{get_bool_by_obj_prop, get_literal_str_by_obj_prop, get_regex_by_obj_prop};
use crate::visitors::{expr_name, JavascriptParser};

fn create_import_meta_context_dependency(node: &CallExpr) -> Option<ImportMetaContextDependency> {
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
        return Some(str.value.to_string());
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
  let reg_str = reg.to_string();
  let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object()) {
    let regexp = get_regex_by_obj_prop(obj, "regExp")
      .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"))
      .unwrap_or(RspackRegex::new(reg).expect("reg failed"));
    // let include = get_regex_by_obj_prop(obj, "include")
    //   .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    // let exclude = get_regex_by_obj_prop(obj, "include")
    //   .map(|regexp| RspackRegex::try_from(regexp).expect("reg failed"));
    let mode = get_literal_str_by_obj_prop(obj, "mode")
      .map(|s| s.value.to_string().as_str().into())
      .unwrap_or(ContextMode::Sync);
    let recursive = get_bool_by_obj_prop(obj, "recursive")
      .map(|bool| bool.value)
      .unwrap_or(true);
    let reg_str = regexp_as_str(&regexp).to_string();
    ContextOptions {
      chunk_name: None,
      reg_exp: clean_regexp_in_context_module(regexp),
      reg_str,
      include: None,
      exclude: None,
      recursive,
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      namespace_object: ContextNameSpaceObject::Unset,
      mode,
      start: node.span().real_lo(),
      end: node.span().real_hi(),
    }
  } else {
    ContextOptions {
      chunk_name: None,
      recursive: true,
      mode: ContextMode::Sync,
      include: None,
      exclude: None,
      reg_exp: context_reg_exp(reg, ""),
      reg_str,
      category: DependencyCategory::Esm,
      request: context.clone(),
      context,
      namespace_object: ContextNameSpaceObject::Unset,
      start: node.span().real_lo(),
      end: node.span().real_hi(),
    }
  };
  Some(ImportMetaContextDependency::new(
    node.span.real_lo(),
    node.span.real_hi(),
    context_options,
    Some(node.span.into()),
  ))
}

pub struct ImportMetaContextDependencyParserPlugin;

impl JavascriptParserPlugin for ImportMetaContextDependencyParserPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if ident == expr_name::IMPORT_META_WEBPACK_CONTEXT {
      Some(eval::evaluate_to_identifier(
        expr_name::IMPORT_META_WEBPACK_CONTEXT.to_string(),
        expr_name::IMPORT_META.to_string(),
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
    if for_name != expr_name::IMPORT_META_WEBPACK_CONTEXT
      || expr.args.is_empty()
      || expr.args.len() > 2
    {
      None
    } else if let Some(dep) = create_import_meta_context_dependency(expr) {
      parser.dependencies.push(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
