use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::ModuleDependency;
use rspack_error::{Diagnostic, DiagnosticKind};
use swc_core::{
  common::pass::AstNodePath,
  css::{
    ast::{ImportHref, ImportPrelude, Stylesheet, Url, UrlValue},
    visit::{AstParentKind, AstParentNodeRef, VisitAstPath, VisitWithPath},
  },
};

use crate::dependency::{CssImportDependency, CssUrlDependency};

static IS_MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^~").expect("TODO:"));

pub fn as_parent_path(ast_path: &AstNodePath<AstParentNodeRef<'_>>) -> Vec<AstParentKind> {
  ast_path.iter().map(|n| n.kind()).collect()
}

pub fn analyze_dependencies(
  ss: &mut Stylesheet,
  code_generation_dependencies: &mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Box<dyn ModuleDependency>> {
  let mut v = Analyzer {
    deps: Vec::new(),
    code_generation_dependencies,
    diagnostics,
  };
  ss.visit_with_path(&mut v, &mut Default::default());

  v.deps
}

#[derive(Debug)]
struct Analyzer<'a> {
  deps: Vec<Box<dyn ModuleDependency>>,
  code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &'a mut Vec<Diagnostic>,
}

fn replace_module_request_prefix(specifier: String, diagnostics: &mut Vec<Diagnostic>) -> String {
  if IS_MODULE_REQUEST.is_match(&specifier) {
    diagnostics.push(
      Diagnostic::warn(
        "Deprecated '~'".to_string(),
        "'@import' or 'url()' with a request starts with '~' is deprecated.".to_string(),
        0,
        0,
      )
      .with_kind(DiagnosticKind::Css),
    );
    IS_MODULE_REQUEST.replace(&specifier, "").to_string()
  } else {
    specifier
  }
}

impl VisitAstPath for Analyzer<'_> {
  fn visit_import_prelude<'ast: 'r, 'r>(
    &mut self,
    n: &'ast ImportPrelude,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    let specifier = match &*n.href {
      ImportHref::Url(u) => u.value.as_ref().map(|box s| match s {
        UrlValue::Str(s) => s.value.to_string(),
        UrlValue::Raw(r) => r.value.to_string(),
      }),
      ImportHref::Str(s) => Some(s.value.to_string()),
    };
    if let Some(specifier) = specifier {
      let specifier = replace_module_request_prefix(specifier, self.diagnostics);
      self.deps.push(Box::new(CssImportDependency::new(
        specifier,
        Some(n.span.into()),
        as_parent_path(ast_path),
      )));
    }
  }

  fn visit_url<'ast: 'r, 'r>(
    &mut self,
    u: &'ast Url,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    u.visit_children_with_path(self, ast_path);

    let specifier = u.value.as_ref().map(|box v| match v {
      UrlValue::Str(s) => s.value.to_string(),
      UrlValue::Raw(r) => r.value.to_string(),
    });
    if let Some(specifier) = specifier {
      let specifier = replace_module_request_prefix(specifier, self.diagnostics);
      let dep = Box::new(CssUrlDependency::new(
        specifier,
        Some(u.span.into()),
        as_parent_path(ast_path),
      ));
      self.deps.push(dep.clone());
      self.code_generation_dependencies.push(dep);
    }
  }
}
