use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{Compilation, Dependency, Module, ModuleDependency, ResolveKind};
use rspack_error::{Diagnostic, DiagnosticKind};
use swc_core::{common::util::take::Take, ecma::atoms::Atom};

use swc_css::{
  ast::{AtRulePrelude, ImportHref, ImportPrelude, Rule, Stylesheet, Url, UrlValue},
  visit::{Visit, VisitMut, VisitMutWith, VisitWith},
};

static IS_MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^?]*~").expect("TODO:"));

pub fn analyze_dependencies(
  ss: &mut Stylesheet,
  code_generation_dependencies: &mut Vec<ModuleDependency>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ModuleDependency> {
  let mut v = Analyzer {
    deps: Vec::new(),
    code_generation_dependencies,
    diagnostics,
  };
  ss.visit_with(&mut v);
  ss.visit_mut_with(&mut RemoveAtImport);
  v.deps
}

#[derive(Debug)]
struct Analyzer<'a> {
  deps: Vec<ModuleDependency>,
  code_generation_dependencies: &'a mut Vec<ModuleDependency>,
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

impl Visit for Analyzer<'_> {
  fn visit_import_prelude(&mut self, n: &ImportPrelude) {
    n.visit_children_with(self);

    let specifier = match &*n.href {
      ImportHref::Url(u) => u.value.as_ref().map(|box s| match s {
        UrlValue::Str(s) => s.value.to_string(),
        UrlValue::Raw(v) => v.value.to_string(),
      }),
      ImportHref::Str(s) => Some(s.value.to_string()),
    };
    if let Some(specifier) = specifier && is_url_requestable(&specifier) {
      let specifier = replace_module_request_prefix(specifier, self.diagnostics);
      self.deps.push(ModuleDependency {
        specifier,
        kind: ResolveKind::AtImport,
        span: Some(n.span.into()),
      });
    }
  }

  fn visit_url(&mut self, u: &Url) {
    u.visit_children_with(self);

    let specifier = u.value.as_ref().map(|box v| match v {
      UrlValue::Str(s) => s.value.to_string(),
      UrlValue::Raw(r) => r.value.to_string(),
    });
    if let Some(specifier) = specifier && is_url_requestable(&specifier) {
      let specifier = replace_module_request_prefix(specifier, self.diagnostics);
      let dep = ModuleDependency {
        specifier,
        kind: ResolveKind::UrlToken,
        span: Some(u.span.into()),
      };
      self.deps.push(dep.clone());
      self.code_generation_dependencies.push(dep);
    }
  }
}

#[derive(Debug, Default)]
struct RemoveAtImport;

impl VisitMut for RemoveAtImport {
  fn visit_mut_stylesheet(&mut self, n: &mut Stylesheet) {
    n.visit_mut_children_with(self);
    n.rules = n
      .rules
      .take()
      .into_iter()
      .filter(|rule| match rule {
        Rule::AtRule(at_rule) => {
          if let Some(box AtRulePrelude::ImportPrelude(prelude)) = &at_rule.prelude {
            let href_string = match &prelude.href {
              box swc_css::ast::ImportHref::Url(url) => {
                let href_string = url
                  .value
                  .as_ref()
                  .map(|box value| match value {
                    UrlValue::Str(str) => str.value.clone(),
                    UrlValue::Raw(raw) => raw.value.clone(),
                  })
                  .unwrap_or_default();
                href_string
              }
              box swc_css::ast::ImportHref::Str(str) => str.value.clone(),
            };
            !is_url_requestable(&href_string)
          } else {
            true
          }
        }
        _ => true,
      })
      .collect();
  }
}

pub fn rewrite_url(ss: &mut Stylesheet, module: &dyn Module, compilation: &Compilation) {
  let mut v = RewriteUrl {
    module,
    compilation,
  };
  ss.visit_mut_with(&mut v);
}

#[derive(Debug)]
struct RewriteUrl<'a> {
  module: &'a dyn Module,
  compilation: &'a Compilation,
}

impl RewriteUrl<'_> {
  pub fn get_target_url(&mut self, specifier: String) -> Option<String> {
    let from = Dependency {
      parent_module_identifier: Some(self.module.identifier()),
      detail: ModuleDependency {
        specifier,
        kind: ResolveKind::UrlToken,
        span: None,
      },
    };
    let from = self.compilation.module_graph.module_by_dependency(&from)?;

    self
      .compilation
      .code_generation_results
      .module_generation_result_map
      .get(&from.module_identifier)
      .and_then(|result| result.data.get("filename"))
      .map(|value| value.to_string())
  }
}

impl VisitMut for RewriteUrl<'_> {
  fn visit_mut_url(&mut self, url: &mut Url) {
    match url.value {
      Some(box UrlValue::Str(ref mut s)) => {
        if !is_url_requestable(&s.value) {
          return;
        }
        if let Some(target) = self.get_target_url(s.value.to_string()) {
          s.raw = Some(Atom::from(target.clone()));
          s.value = target.into();
        }
      }
      Some(box UrlValue::Raw(ref mut s)) => {
        if !is_url_requestable(&s.value) {
          return;
        }
        if let Some(target) = self.get_target_url(s.value.to_string()) {
          s.raw = Some(Atom::from(target.clone()));
          s.value = target.into();
        }
      }
      None => {}
    }
  }
}

fn is_url_requestable(url: &str) -> bool {
  !url.starts_with('#') && !rspack_core::should_skip_resolve(url)
}
