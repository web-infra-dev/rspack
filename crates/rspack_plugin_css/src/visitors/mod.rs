use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{BoxDependency, ModuleDependency, SpanExt};
use rspack_error::miette::{Diagnostic, Severity};
use rspack_error::{DiagnosticExt, TraceableError};
use swc_core::common::{FileName, Span};
use swc_core::css::ast::{
  AtRule, AtRuleName, Function, ImportHref, ImportPrelude, Stylesheet, Token, TokenAndSpan, Url,
  UrlValue,
};
use swc_core::css::visit::{Visit, VisitWith};

use crate::{
  dependency::{CssImportDependency, CssUrlDependency},
  utils::normalize_url,
};

static IS_MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^~").expect("TODO:"));

pub fn analyze_dependencies(
  ss: &Stylesheet,
  code_generation_dependencies: &mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  source_code: &str,
  filename: &str,
) -> Vec<BoxDependency> {
  let mut v = Analyzer {
    deps: Vec::new(),
    code_generation_dependencies,
    diagnostics,
    nearest_at_import_span: None,
    url_function_span: None,
    source_code,
    filename,
    // in_support_contdition: false,
  };
  ss.visit_with(&mut v);

  v.deps
}

#[derive(Debug)]
struct Analyzer<'a> {
  deps: Vec<BoxDependency>,
  code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  diagnostics: &'a mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  nearest_at_import_span: Option<Span>,
  url_function_span: Option<Span>,
  source_code: &'a str,
  filename: &'a str,
  // in_support_contdition: bool,
}

fn replace_module_request_prefix(
  specifier: String,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  source_code: &str,
  filename: &str,
  span: Span,
) -> String {
  if IS_MODULE_REQUEST.is_match(&specifier) {
    let cm: Arc<swc_core::common::SourceMap> = Default::default();
    let fm = cm.new_source_file(
      FileName::Custom(filename.to_string()),
      source_code.to_string(),
    );
    let start = span.real_lo() as usize;
    let end = span.real_hi() as usize;
    let import_statement = &source_code[start..end];
    let mat = Regex::new(r"~")
      .expect("Failed to compile the regex.")
      .find(import_statement)
      .expect("Failed to find the '~'.");
    diagnostics.push(
      TraceableError::from_source_file(
        &fm.clone(),
        start + mat.start(),
        start + mat.end(),
        "CSS parsing warning".to_string(),
        "'@import' or 'url()' with a request starts with '~' is deprecated.".to_string(),
      )
      .with_help(Some("Remove '~' from the request."))
      .with_severity(Severity::Warning)
      .boxed(),
    );
    IS_MODULE_REQUEST.replace(&specifier, "").to_string()
  } else {
    specifier
  }
}

static URL_KEYWORD: &str = "url";
static IMPORT_KEYWORD: &str = "import";

impl Analyzer<'_> {
  fn analyze_url(&mut self, value: impl Into<String>, span: Span) {
    let mut specifier = replace_module_request_prefix(
      value.into(),
      self.diagnostics,
      self.source_code,
      self.filename,
      span,
    );
    specifier = normalize_url(&specifier);
    let dep = Box::new(CssUrlDependency::new(
      specifier,
      Some(span.into()),
      span.real_lo(),
      span.real_hi(),
    ));
    self.deps.push(dep.clone());
    self.code_generation_dependencies.push(dep);
  }
}

impl Visit for Analyzer<'_> {
  fn visit_at_rule(&mut self, n: &AtRule) {
    if let AtRuleName::Ident(ident) = &n.name
      && &*ident.value == IMPORT_KEYWORD
    {
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
        UrlValue::Str(s) => s.value.to_string(),
        UrlValue::Raw(r) => r.value.to_string(),
      }),
      ImportHref::Str(s) => Some(s.value.to_string()),
    };
    if let Some(specifier) = specifier {
      let specifier = replace_module_request_prefix(
        specifier,
        self.diagnostics,
        self.source_code,
        self.filename,
        span,
      );
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

  /// Handle `url("...")` in css variables.
  ///
  ///  Function {
  ///      span: Span { ... },
  ///      name: Ident(
  ///          Ident {
  ///              span: Span { ... },
  ///              value: "url",
  ///              raw: Some(
  ///                  "url",
  ///              ),
  ///          },
  ///      ),
  ///      value: [
  ///          PreservedToken(
  ///              TokenAndSpan {
  ///                  span: Span { ... },
  ///                  token: String {
  ///                      value: "...",
  ///                      raw: ""..."",
  ///                  },
  ///              },
  ///          ),
  ///      ],
  ///  }
  fn visit_function(&mut self, f: &Function) {
    if let Some(i) = f.name.as_ident()
      && i.value == URL_KEYWORD
    {
      self.url_function_span = Some(f.span);
      f.visit_children_with(self);
      self.url_function_span = None;
    } else {
      f.visit_children_with(self);
    }
  }

  fn visit_token_and_span(&mut self, t: &TokenAndSpan) {
    match &t.token {
      Token::Url { value, .. } => self.analyze_url(value.as_ref(), t.span),
      Token::String { value, .. } if let Some(span) = self.url_function_span => {
        self.analyze_url(value.as_ref(), span)
      }
      _ => t.visit_children_with(self),
    }
  }

  fn visit_url(&mut self, u: &Url) {
    u.visit_children_with(self);
    // Wait for @supports
    // if !self.in_support_contdition {
    let specifier = u.value.as_ref().map(|box v| match v {
      UrlValue::Str(s) => s.value.to_string(),
      UrlValue::Raw(r) => r.value.to_string(),
    });
    if let Some(specifier) = specifier
      && !specifier.is_empty()
    {
      self.analyze_url(specifier, u.span);
    }
  }
}
