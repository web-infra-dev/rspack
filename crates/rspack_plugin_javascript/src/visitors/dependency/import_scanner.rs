use once_cell::sync::Lazy;
use rspack_core::{ContextMode, ContextOptions, DependencyCategory, ModuleDependency, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::{
  common::{comments::Comments, Span},
  ecma::{
    ast::{CallExpr, Callee, Expr, Lit},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::scanner::scanner_context_module;
use crate::dependency::{ImportContextDependency, ImportDependency};

pub struct ImportScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub comments: Option<&'a dyn Comments>,
}

impl<'a> ImportScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    comments: Option<&'a dyn Comments>,
  ) -> Self {
    Self {
      dependencies,
      comments,
    }
  }

  fn try_extract_webpack_chunk_name(&self, first_arg_span_of_import_call: &Span) -> Option<String> {
    use swc_core::common::comments::CommentKind;
    static WEBPACK_CHUNK_NAME_CAPTURE_RE: Lazy<regex::Regex> = Lazy::new(|| {
      regex::Regex::new(r#"webpackChunkName\s*:\s*("(?P<_1>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)"|'(?P<_2>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)'|`(?P<_3>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)`)"#)
        .expect("invalid regex")
    });
    self
      .comments
      .with_leading(first_arg_span_of_import_call.lo, |comments| {
        let ret = comments
          .iter()
          .rev()
          .filter(|c| matches!(c.kind, CommentKind::Block))
          .find_map(|comment| {
            WEBPACK_CHUNK_NAME_CAPTURE_RE
              .captures(&comment.text)
              .and_then(|captures| {
                if let Some(cap) = captures.name("_1") {
                  Some(cap)
                } else if let Some(cap) = captures.name("_2") {
                  Some(cap)
                } else {
                  captures.name("_3")
                }
              })
              .map(|mat| mat.as_str().to_string())
          });
        ret
      })
  }
}

impl Visit for ImportScanner<'_> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, node: &CallExpr) {
    if let Callee::Import(import_call) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          match dyn_imported.expr.as_ref() {
            Expr::Lit(Lit::Str(imported)) => {
              let chunk_name = self.try_extract_webpack_chunk_name(&imported.span);
              self.dependencies.push(Box::new(ImportDependency::new(
                node.span.real_lo(),
                node.span.real_hi(),
                imported.value.clone(),
                Some(node.span.into()),
                chunk_name,
              )));
            }
            Expr::Tpl(tpl) if tpl.quasis.len() == 1 => {
              let chunk_name = self.try_extract_webpack_chunk_name(&tpl.span);
              let request = JsWord::from(
                tpl
                  .quasis
                  .first()
                  .expect("should have one quasis")
                  .raw
                  .to_string(),
              );
              self.dependencies.push(Box::new(ImportDependency::new(
                node.span.real_lo(),
                node.span.real_hi(),
                request,
                Some(node.span.into()),
                chunk_name,
              )));
            }
            _ => {
              if let Some((context, reg)) = scanner_context_module(dyn_imported.expr.as_ref()) {
                self
                  .dependencies
                  .push(Box::new(ImportContextDependency::new(
                    import_call.span.real_lo(),
                    import_call.span.real_hi(),
                    node.span.real_hi(),
                    ContextOptions {
                      mode: ContextMode::Lazy,
                      recursive: true,
                      reg_exp: RspackRegex::new(&reg).expect("reg failed"),
                      reg_str: reg,
                      include: None,
                      exclude: None,
                      category: DependencyCategory::Esm,
                      request: context,
                    },
                    Some(node.span.into()),
                  )));
              }
            }
          }
        }
      }
    } else {
      node.visit_children_with(self);
    }
  }
}
