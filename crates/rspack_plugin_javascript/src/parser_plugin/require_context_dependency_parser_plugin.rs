use rspack_core::{clean_regexp_in_context_module, try_convert_str_to_context_mode};
use rspack_core::{ContextMode, ContextOptions, DependencyCategory, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::common::Spanned;
use swc_core::ecma::ast::CallExpr;

use super::JavascriptParserPlugin;
use crate::dependency::RequireContextDependency;
use crate::visitors::expr_matcher::is_require_context;
use crate::visitors::JavascriptParser;

pub struct RequireContextDependencyParserPlugin;

const DEFAULT_REGEXP_STR: &str = r"^\.\/.*$";

impl JavascriptParserPlugin for RequireContextDependencyParserPlugin {
  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, _name: &str) -> Option<bool> {
    if expr
      .callee
      .as_expr()
      .map_or(true, |expr| !is_require_context(&**expr))
    {
      return None;
    }

    let mode = if expr.args.len() == 4 {
      let mode_expr = parser.evaluate_expression(&expr.args[3].expr);
      if !mode_expr.is_string() {
        // FIXME: return `None` in webpack
        ContextMode::Sync
      } else if let Some(mode_expr) = try_convert_str_to_context_mode(mode_expr.string()) {
        mode_expr
      } else {
        ContextMode::Sync
      }
    } else {
      ContextMode::Sync
    };

    let reg_exp = if expr.args.len() >= 3 {
      let reg_exp_expr = parser.evaluate_expression(&expr.args[2].expr);
      if !reg_exp_expr.is_regexp() {
        // FIXME: return `None` in webpack
        RspackRegex::new(DEFAULT_REGEXP_STR).expect("reg should success")
      } else {
        let (expr, flags) = reg_exp_expr.regexp();
        RspackRegex::with_flags(expr.as_str(), flags.as_str()).expect("reg should success")
      }
    } else {
      RspackRegex::new(DEFAULT_REGEXP_STR).expect("reg should success")
    };

    let recursive = if expr.args.len() >= 2 {
      let recursive_expr = parser.evaluate_expression(&expr.args[1].expr);
      if !recursive_expr.is_bool() {
        // FIXME: return `None` in webpack
        true
      } else {
        recursive_expr.bool()
      }
    } else {
      true
    };

    if !expr.args.is_empty() {
      let request_expr = parser.evaluate_expression(&expr.args[0].expr);
      if !request_expr.is_string() {
        return None;
      }

      parser
        .dependencies
        .push(Box::new(RequireContextDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          ContextOptions {
            mode,
            recursive,
            reg_exp: clean_regexp_in_context_module(reg_exp),
            include: None,
            exclude: None,
            category: DependencyCategory::CommonJS,
            request: request_expr.string().to_string(),
            context: request_expr.string().to_string(),
            namespace_object: rspack_core::ContextNameSpaceObject::Unset,
            group_options: None,
            start: expr.span().real_lo(),
            end: expr.span().real_hi(),
          },
          Some(expr.span.into()),
        )));
      return Some(true);
    }

    None
  }
}
