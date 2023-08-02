use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{ModuleDependency, SpanExt};
use rspack_error::{Diagnostic, DiagnosticKind};
use swc_core::common::Span;
use swc_core::css::ast::{
  AtRule, AtRuleName, ImportHref, ImportPrelude, Stylesheet, Url, UrlValue,
};
use swc_core::css::visit::{Visit, VisitWith};
use swc_core::ecma::atoms::JsWord;

use crate::{
  dependency::{CssImportDependency, CssUrlDependency},
  utils::normalize_url,
};

static IS_MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^~").expect("TODO:"));

pub fn analyze_dependencies(
  ss: &Stylesheet,
  code_generation_dependencies: &mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Box<dyn ModuleDependency>> {
  let mut v = Analyzer {
    deps: Vec::new(),
    code_generation_dependencies,
    diagnostics,
    nearest_at_import_span: None,
    // in_support_contdition: false,
  };
  ss.visit_with(&mut v);

  v.deps
}

#[derive(Debug)]
struct Analyzer<'a> {
  deps: Vec<Box<dyn ModuleDependency>>,
  code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &'a mut Vec<Diagnostic>,
  nearest_at_import_span: Option<Span>,
  // in_support_contdition: bool,
}

fn replace_module_request_prefix(specifier: JsWord, diagnostics: &mut Vec<Diagnostic>) -> JsWord {
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
    IS_MODULE_REQUEST.replace(&specifier, "").into()
  } else {
    specifier
  }
}

impl Visit for Analyzer<'_> {
  fn visit_at_rule(&mut self, n: &AtRule) {
    if let AtRuleName::Ident(ident) = &n.name && &*ident.value == "import" {
      self.nearest_at_import_span = Some(n.span);
    }
    n.visit_children_with(self);
  }

  fn visit_import_prelude(&mut self, n: &ImportPrelude) {
    let Some(span) = self.nearest_at_import_span.take() else {
      return;
    };

    let specifier = match &*n.href {
      ImportHref::Url(u) => u.value.as_ref().map(|box s| match s {
        UrlValue::Str(s) => s.value.clone(),
        UrlValue::Raw(r) => r.value.clone(),
      }),
      ImportHref::Str(s) => Some(s.value.clone()),
    };
    if let Some(specifier) = specifier {
      let specifier = replace_module_request_prefix(specifier, self.diagnostics);
      self.deps.push(Box::new(CssImportDependency::new(
        specifier,
        Some(span.into()),
        span.real_lo(),
        span.real_hi(),
      )));
    }
  }

  // Wait for @supports
  // fn visit_supports_condition<'ast: 'r, 'r>(
  //   &mut self,
  //   n: &'ast swc_core::css::ast::SupportsCondition,
  //   ast_path: &mut swc_core::css::visit::AstNodePath<'r>,
  // ) {
  //   self.in_support_contdition = true;
  //   n.visit_children_with_path(self, ast_path);
  //   self.in_support_contdition = false;
  // }

  fn visit_url(&mut self, u: &Url) {
    u.visit_children_with(self);
    // Wait for @supports
    // if !self.in_support_contdition {
    let specifier = u.value.as_ref().map(|box v| match v {
      UrlValue::Str(s) => s.value.clone(),
      UrlValue::Raw(r) => r.value.clone(),
    });
    if let Some(specifier) = specifier && !specifier.is_empty(){
    let mut specifier = replace_module_request_prefix(specifier, self.diagnostics);
    specifier = normalize_url(&specifier).into();
    let dep = Box::new(CssUrlDependency::new(
      specifier,
      Some(u.span.into()),
      u.span.real_lo(),
      u.span.real_hi()
    ));
    self.deps.push(dep.clone());
    self.code_generation_dependencies.push(dep);
  // }
    }
  }
}
