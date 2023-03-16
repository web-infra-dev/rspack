mod analyze_imports_with_path;
pub use analyze_imports_with_path::*;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  Compilation, CssImportDependency, CssUrlDependency, Dependency, DependencyType, Module,
  ModuleDependency, ModuleGraphModule, ModuleIdentifier,
};
use rspack_error::{Diagnostic, DiagnosticKind};
use swc_core::{
  common::{pass::AstNodePath, util::take::Take},
  css::{
    ast::{AtRulePrelude, ImportHref, ImportPrelude, Rule, Stylesheet, Url, UrlValue},
    visit::{
      AstKindPath, AstParentKind, AstParentNodeRef, VisitMut, VisitMutAstPath, VisitMutWith,
      VisitMutWithPath,
    },
  },
};

static IS_MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^?]*~").expect("TODO:"));

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
  ss.visit_mut_with_path(&mut v, &mut Default::default());
  // TODO: use dependency to remove at import
  ss.visit_mut_with(&mut RemoveAtImport);

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

impl VisitMutAstPath for Analyzer<'_> {
  fn visit_mut_import_prelude(&mut self, n: &mut ImportPrelude, ast_path: &mut AstKindPath) {
    n.visit_mut_children_with_path(self, ast_path);

    let specifier = match &mut *n.href {
      ImportHref::Url(u) => u.value.as_mut().map(|box s| match s {
        UrlValue::Str(s) => {
          let replaced = replace_module_request_prefix(s.value.to_string(), self.diagnostics);
          s.value = replaced.into();
          s.raw = None;
          s.value.to_string()
        }
        UrlValue::Raw(s) => {
          let replaced = replace_module_request_prefix(s.value.to_string(), self.diagnostics);
          s.value = replaced.into();
          s.raw = None;
          s.value.to_string()
        }
      }),
      ImportHref::Str(s) => {
        let replaced = replace_module_request_prefix(s.value.to_string(), self.diagnostics);
        s.value = replaced.into();
        s.raw = None;
        Some(s.value.to_string())
      }
    };
    if let Some(specifier) = specifier && is_url_requestable(&specifier) {
      self.deps.push(box CssImportDependency::new(specifier, Some(n.span.into()), ast_path.iter().copied().collect()));
    }
  }

  fn visit_mut_url(&mut self, u: &mut Url, ast_path: &mut AstKindPath) {
    u.visit_mut_children_with_path(self, ast_path);

    let specifier = u.value.as_mut().map(|box v| match v {
      UrlValue::Str(s) => {
        let replaced = replace_module_request_prefix(s.value.to_string(), self.diagnostics);
        s.value = replaced.into();
        s.raw = None;
        s.value.to_string()
      }
      UrlValue::Raw(s) => {
        let replaced = replace_module_request_prefix(s.value.to_string(), self.diagnostics);
        s.value = replaced.into();
        s.raw = None;
        s.value.to_string()
      }
    });
    if let Some(specifier) = specifier && is_url_requestable(&specifier) {
      let dep = box CssUrlDependency::new(specifier, Some(u.span.into()), ast_path.iter().copied().collect());
      // TODO avoid dependency clone
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
              box swc_core::css::ast::ImportHref::Url(url) => {
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
              box swc_core::css::ast::ImportHref::Str(str) => str.value.clone(),
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
  // TODO: Workaround. Remove this in the future
  fn resolve_module_legacy(
    &self,
    module_identifier: &ModuleIdentifier,
    src: &str,
    dependency_type: &DependencyType,
  ) -> Option<&ModuleGraphModule> {
    self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| {
        mgm.dependencies.iter().find_map(|id| {
          let dependency = self
            .compilation
            .module_graph
            .dependency_by_id(id)
            .expect("should have dependency");
          if dependency.request() == src && dependency.dependency_type() == dependency_type {
            self
              .compilation
              .module_graph
              .module_graph_module_by_dependency_id(id)
          } else {
            None
          }
        })
      })
  }

  pub fn get_target_url(&mut self, specifier: String) -> Option<String> {
    let from = self.resolve_module_legacy(
      &self.module.identifier(),
      &specifier,
      &DependencyType::CssUrl,
    )?;

    let module = self
      .compilation
      .code_generation_results
      .module_generation_result_map
      .get(&from.module_identifier);
    if let Some(module) = module {
      if let Some(url) = module.data.get("url") {
        Some(url.to_string())
      } else if let Some(filename) = module.data.get("filename") {
        let public_path = self
          .compilation
          .options
          .output
          .public_path
          .render(self.compilation, filename);
        Some(format!("{public_path}{filename}"))
      } else {
        None
      }
    } else {
      Some("data:,".to_string())
    }
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
          s.raw = None;
          s.value = target.into();
        }
      }
      Some(box UrlValue::Raw(ref mut s)) => {
        if !is_url_requestable(&s.value) {
          return;
        }
        if let Some(target) = self.get_target_url(s.value.to_string()) {
          s.raw = None;
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
