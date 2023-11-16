use rspack_core::{clean_regexp_in_context_module, BoxDependency, ContextMode};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::ecma::{
  ast::{CallExpr, Lit},
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::is_require_context_call;
use crate::dependency::RequireContextDependency;

pub struct RequireContextScanner<'a> {
  pub dependencies: &'a mut Vec<BoxDependency>,
}

impl<'a> RequireContextScanner<'a> {
  pub fn new(dependencies: &'a mut Vec<BoxDependency>) -> Self {
    Self { dependencies }
  }
}

impl Visit for RequireContextScanner<'_> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, node: &CallExpr) {
    if is_require_context_call(node) && !node.args.is_empty() {
      let mut analyze = |request: &str| {
        let recursive =
          if let Some(Lit::Bool(bool)) = node.args.get(1).and_then(|x| x.expr.as_lit()) {
            bool.value
          } else {
            true
          };

        let (reg_exp, reg_str) =
          if let Some(Lit::Regex(regex)) = node.args.get(2).and_then(|x| x.expr.as_lit()) {
            (
              RspackRegex::try_from(regex).expect("reg failed"),
              format!("{}|{}", regex.exp, regex.flags),
            )
          } else {
            (
              RspackRegex::new(r"^\.\/.*$").expect("reg failed"),
              r"^\.\/.*$".to_string(),
            )
          };

        let mode = if let Some(Lit::Str(str)) = node.args.get(3).and_then(|x| x.expr.as_lit()) {
          str.value.to_string().as_str().into()
        } else {
          ContextMode::Sync
        };
        self
          .dependencies
          .push(Box::new(RequireContextDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            ContextOptions {
              chunk_name: None,
              mode,
              recursive,
              reg_exp: clean_regexp_in_context_module(reg_exp),
              reg_str,
              include: None,
              exclude: None,
              category: DependencyCategory::CommonJS,
              request: request.to_string(),
              namespace_object: ContextNameSpaceObject::Unset,
            },
            Some(node.span.into()),
          )));
      };

      // Handle string evaluatables.
      // For example: require.context("", ...), require.context(``, ...)
      // https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/dependencies/RequireContextDependencyParserPlugin.js#L47
      // TODO: should've used expression evaluation to handle cases like `abc${"efg"}`, etc.
      match node.args.first().map(|x| &*x.expr) {
        Some(t) if let Some(Lit::Str(str)) = t.as_lit() => analyze(&str.value),
        Some(t)
          if let Some(tpl) = t.as_tpl()
            && tpl.exprs.is_empty()
            && tpl.quasis.len() == 1
            && let Some(el) = tpl.quasis.first() =>
        {
          analyze(&el.raw)
        }
        _ => (),
      }
    } else {
      node.visit_children_with(self);
    }
  }
}
