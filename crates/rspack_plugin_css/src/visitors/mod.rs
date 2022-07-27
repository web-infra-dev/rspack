pub mod css_assets;

use rspack_core::{ModuleDependency, ResolveKind};
use swc_common::util::take::Take;
use swc_css::{
  ast::{AtRulePrelude, Rule, Stylesheet, Url, UrlValue},
  visit::{VisitMut, VisitMutWith},
};

#[derive(Debug, Default)]
pub struct DependencyScanner {
  pub dependencies: Vec<ModuleDependency>,
}

impl VisitMut for DependencyScanner {
  fn visit_mut_url(&mut self, n: &mut Url) {
    let ident = &n.name.value;
    if ident == "url" {
      if let Some(UrlValue::Str(str)) = &mut n.value {
        self.dependencies.push(ModuleDependency {
          specifier: str.value.to_string(),
          kind: ResolveKind::UrlToken,
        })
      }
    }
  }
  fn visit_mut_stylesheet(&mut self, n: &mut Stylesheet) {
    n.visit_mut_children_with(self);
    n.rules = n
      .rules
      .take()
      .into_iter()
      .filter(|rule| match rule {
        Rule::AtRule(at_rule) => {
          if let Some(AtRulePrelude::ImportPrelude(prelude)) = &at_rule.prelude {
            let href_string = match &prelude.href {
              swc_css::ast::ImportPreludeHref::Url(url) => url
                .value
                .as_ref()
                .map(|value| match value {
                  UrlValue::Str(str) => str.value.to_string(),
                  UrlValue::Raw(raw) => raw.value.to_string(),
                })
                .unwrap_or_default(),
              swc_css::ast::ImportPreludeHref::Str(str) => str.value.to_string(),
            };
            self.dependencies.push(ModuleDependency {
              specifier: href_string,
              kind: ResolveKind::AtImport,
            });
            false
          } else {
            true
          }
        }
        _ => true,
      })
      .collect();
  }
}
