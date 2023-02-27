use rspack_core::CssAstPath;
use swc_core::common::pass::AstNodePath;
use swc_core::css::ast::{
  ComponentValue, Declaration, DeclarationName, Ident, ImportHref, ImportPrelude, Stylesheet,
  UrlValue,
};
use swc_core::css::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::ecma::atoms::{js_word, JsWord};

use super::as_parent_path;

pub fn analyze_imports_with_path(ss: &Stylesheet) -> Vec<(JsWord, CssAstPath)> {
  let mut v = Analyzer {
    imports: Default::default(),
  };
  ss.visit_with_path(&mut v, &mut Default::default());
  v.imports.sort_unstable();
  v.imports.dedup();
  v.imports
}

struct Analyzer {
  imports: Vec<(JsWord, CssAstPath)>,
}

impl VisitAstPath for Analyzer {
  fn visit_import_prelude<'ast: 'r, 'r>(
    &mut self,
    n: &'ast ImportPrelude,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    n.visit_children_with_path(self, ast_path);

    match &*n.href {
      ImportHref::Url(u) => {
        if let Some(s) = &u.value {
          match &**s {
            UrlValue::Str(s) => {
              self
                .imports
                .push((s.value.clone(), as_parent_path(ast_path)));
            }
            UrlValue::Raw(v) => {
              self
                .imports
                .push((v.value.clone(), as_parent_path(ast_path)));
            }
          }
        }
      }
      ImportHref::Str(s) => {
        self
          .imports
          .push((s.value.clone(), as_parent_path(ast_path)));
      }
    }
  }

  fn visit_declaration<'ast: 'r, 'r>(
    &mut self,
    d: &'ast Declaration,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    d.visit_children_with_path(self, ast_path);

    if let DeclarationName::Ident(name) = &d.name {
      if &*name.value == "composes" {
        // composes: name from 'foo.css'
        if d.value.len() >= 3 {
          if let (
            ComponentValue::Ident(box Ident {
              value: js_word!("from"),
              ..
            }),
            ComponentValue::Str(s),
          ) = (&d.value[d.value.len() - 2], &d.value[d.value.len() - 1])
          {
            self
              .imports
              .push((s.value.clone(), as_parent_path(ast_path)));
          }
        }
      }
    }
  }
}
