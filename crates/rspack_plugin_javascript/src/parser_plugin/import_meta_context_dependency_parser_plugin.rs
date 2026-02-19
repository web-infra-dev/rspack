use rspack_core::{ContextMode, ContextNameSpaceObject, ContextOptions, DependencyCategory};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{CallExpr, GetSpan, Lit};

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
  node: CallExpr,
  parser: &mut JavascriptParser,
) -> Option<ImportMetaContextDependency> {
  assert!(node.callee(&parser.ast).is_expr());
  let dyn_imported = parser
    .ast
    .get_node_in_sub_range(node.args(&parser.ast).first()?);
  if dyn_imported.spread(&parser.ast).is_some() {
    return None;
  }
  let context = dyn_imported
    .expr(&parser.ast)
    .as_lit()
    .and_then(|lit| {
      if let Lit::Str(str) = lit {
        return Some(
          parser
            .ast
            .get_wtf8(str.value(&parser.ast))
            .to_string_lossy()
            .to_string(),
        );
      }
      None
    })
    // TODO: should've used expression evaluation to handle cases like `abc${"efg"}`, etc.
    .or_else(|| {
      if let Some(tpl) = dyn_imported.expr(&parser.ast).as_tpl()
        && tpl.exprs(&parser.ast).is_empty()
        && tpl.quasis(&parser.ast).len() == 1
        && let Some(el) = tpl.quasis(&parser.ast).first()
      {
        let el = parser.ast.get_node_in_sub_range(el);
        return Some(parser.ast.get_utf8(el.raw(&parser.ast)).to_string());
      }
      None
    })?;
  let reg = r"^\.\/.*$";
  let context_options = if let Some(obj) = node.args(&parser.ast).get(1).and_then(|arg| {
    parser
      .ast
      .get_node_in_sub_range(arg)
      .expr(&parser.ast)
      .as_object()
  }) {
    let regexp = get_regex_by_obj_prop(&parser.ast, obj, "regExp");
    let regexp_span = regexp.map(|r| r.span(&parser.ast).into());
    let regexp = regexp.map_or(RspackRegex::new(reg).expect("reg failed"), |regexp| {
      RspackRegex::try_from_swc_regex(&parser.ast, regexp).expect("reg failed")
    });
    let include = get_regex_by_obj_prop(&parser.ast, obj, "include")
      .map(|regexp| RspackRegex::try_from_swc_regex(&parser.ast, regexp).expect("reg failed"));
    let exclude = get_regex_by_obj_prop(&parser.ast, obj, "exclude")
      .map(|regexp| RspackRegex::try_from_swc_regex(&parser.ast, regexp).expect("reg failed"));
    let mode =
      get_literal_str_by_obj_prop(&parser.ast, obj, "mode").map_or(ContextMode::Sync, |s| {
        parser
          .ast
          .get_wtf8(s.value(&parser.ast))
          .to_string_lossy()
          .as_ref()
          .into()
      });
    let recursive = get_bool_by_obj_prop(&parser.ast, obj, "recursive")
      .is_none_or(|bool| bool.value(&parser.ast));
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
      start: node.span(&parser.ast).real_lo(),
      end: node.span(&parser.ast).real_hi(),
      referenced_exports: None,
      attributes: None,
      phase: None,
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
      start: node.span(&parser.ast).real_lo(),
      end: node.span(&parser.ast).real_hi(),
      referenced_exports: None,
      attributes: None,
      phase: None,
    }
  };
  Some(ImportMetaContextDependency::new(
    context_options,
    node.span(&parser.ast).into(),
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
  ) -> Option<BasicEvaluatedExpression> {
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

  fn call(&self, parser: &mut JavascriptParser, expr: CallExpr, for_name: &str) -> Option<bool> {
    if for_name != expr_name::IMPORT_META_CONTEXT
      || expr.args(&parser.ast).is_empty()
      || expr.args(&parser.ast).len() > 2
    {
      None
    } else if let Some(dep) = create_import_meta_context_dependency(expr, parser) {
      parser.add_dependency(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
