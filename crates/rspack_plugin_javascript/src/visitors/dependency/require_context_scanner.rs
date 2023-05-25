use rspack_core::{ContextMode, ContextOptions, DependencyCategory, ModuleDependency, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::ecma::{
  ast::{CallExpr, Lit},
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::is_require_context_call;
use crate::dependency::RequireContextDependency;

pub struct RequireContextScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
}

impl<'a> RequireContextScanner<'a> {
  pub fn new(dependencies: &'a mut Vec<Box<dyn ModuleDependency>>) -> Self {
    Self { dependencies }
  }
}

impl Visit for RequireContextScanner<'_> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, node: &CallExpr) {
    if is_require_context_call(node) && !node.args.is_empty() {
      if let Some(Lit::Str(str)) = node.args.get(0).and_then(|x| x.expr.as_lit()) {
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
          match str.value.to_string().as_str() {
            "sync" => ContextMode::Sync,
            "eager" => ContextMode::Eager,
            "weak" => ContextMode::Weak,
            "lazy" => ContextMode::Lazy,
            "lazy-once" => ContextMode::LazyOnce,
            // TODO should give warning
            _ => unreachable!("unknown context mode"),
          }
        } else {
          ContextMode::Sync
        };
        self
          .dependencies
          .push(Box::new(RequireContextDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            ContextOptions {
              mode,
              recursive,
              reg_exp,
              reg_str,
              include: None,
              exclude: None,
              category: DependencyCategory::CommonJS,
              request: str.value.to_string(),
            },
            Some(node.span.into()),
          )));
      }
    } else {
      node.visit_children_with(self);
    }
  }
}
