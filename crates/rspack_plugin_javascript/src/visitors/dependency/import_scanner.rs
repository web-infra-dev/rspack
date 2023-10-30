use once_cell::sync::Lazy;
use rspack_core::{clean_regexp_in_context_module, context_reg_exp};
use rspack_core::{BoxDependency, BuildMeta, ChunkGroupOptions, ContextMode};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::common::comments::Comments;
use swc_core::common::Span;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Lit};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::context_helper::scanner_context_module;
use super::is_import_meta_context_call;
use crate::dependency::{ImportContextDependency, ImportDependency, ImportMetaContextDependency};
use crate::utils::{get_bool_by_obj_prop, get_literal_str_by_obj_prop, get_regex_by_obj_prop};

pub struct ImportScanner<'a> {
  pub dependencies: &'a mut Vec<BoxDependency>,
  pub comments: Option<&'a dyn Comments>,
  pub build_meta: &'a BuildMeta,
}

impl<'a> ImportScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    comments: Option<&'a dyn Comments>,
    build_meta: &'a BuildMeta,
  ) -> Self {
    Self {
      dependencies,
      comments,
      build_meta,
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
    if let Callee::Import(import_call) = &node.callee {
      if let Some(dyn_imported) = node.args.first() {
        if dyn_imported.spread.is_none() {
          match dyn_imported.expr.as_ref() {
            Expr::Lit(Lit::Str(imported)) => {
              let chunk_name = self.try_extract_webpack_chunk_name(&imported.span);
              self.dependencies.push(Box::new(ImportDependency::new(
                node.span.real_lo(),
                node.span.real_hi(),
                imported.value.clone(),
                Some(node.span.into()),
                ChunkGroupOptions::default().name_optional(chunk_name),
                // TODO scan dynamic import referenced exports
                None,
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
                ChunkGroupOptions::default().name_optional(chunk_name),
                None,
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
                      reg_exp: context_reg_exp(&reg, ""),
                      reg_str: reg,
                      include: None,
                      exclude: None,
                      category: DependencyCategory::Esm,
                      request: context,
                      namespace_object: if self.build_meta.strict_harmony_module {
                        ContextNameSpaceObject::Strict
                      } else {
                        ContextNameSpaceObject::Bool(true)
                      },
                    },
                    Some(node.span.into()),
                  )));
              }
            }
          }
        }
      }
    } else if is_import_meta_context_call(node) && !node.args.is_empty() {
      assert!(node.callee.is_expr());
      let Some(dyn_imported) = node.args.first() else {
        return;
      };
      if dyn_imported.spread.is_some() {
        return;
      }
      let Some(lit) = dyn_imported.expr.as_lit() else {
        return;
      };
      let context = match lit {
        Lit::Str(str) => str.value.to_string(),
        _ => return,
      };
      let reg = r"^\.\/.*$";
      let reg_str = reg.to_string();
      let context_options = if let Some(obj) = node.args.get(1).and_then(|arg| arg.expr.as_object())
      {
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
        let reg_str = regexp.raw().to_string();
        ContextOptions {
          reg_exp: clean_regexp_in_context_module(regexp),
          reg_str,
          include: None,
          exclude: None,
          recursive,
          category: DependencyCategory::Esm,
          request: context,
          namespace_object: ContextNameSpaceObject::Unset,
          mode,
        }
      } else {
        ContextOptions {
          recursive: true,
          mode: ContextMode::Sync,
          include: None,
          exclude: None,
          reg_exp: context_reg_exp(reg, ""),
          reg_str,
          category: DependencyCategory::Esm,
          request: context,
          namespace_object: ContextNameSpaceObject::Unset,
        }
      };

      self
        .dependencies
        .push(Box::new(ImportMetaContextDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          context_options,
          Some(node.span.into()),
        )))
    } else {
      node.visit_children_with(self);
    }
  }
}
