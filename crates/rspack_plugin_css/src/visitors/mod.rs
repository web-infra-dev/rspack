pub mod css_assets;

use rspack_core::{ModuleDependency, ResolveKind};
use swc_css::{
  ast::{AtRulePrelude, Rule, Stylesheet, Url, UrlValue},
  visit::{VisitMut, VisitMutWith},
};

#[derive(Debug, Default)]
pub struct DependencyScanner {
  pub dependecies: Vec<ModuleDependency>,
}

impl VisitMut for DependencyScanner {
  fn visit_mut_url(&mut self, n: &mut Url) {
    let ident = &n.name.value;
    if ident == "url" {
      if let Some(UrlValue::Str(str)) = &mut n.value {
        self.dependecies.push(ModuleDependency {
          specifier: str.value.to_string(),
          kind: ResolveKind::UrlToken,
        })
      }
    }
  }
  fn visit_mut_stylesheet(&mut self, n: &mut Stylesheet) {
    n.visit_mut_children_with(self);
    let rules = std::mem::take(&mut n.rules);
    n.rules = rules
      .into_iter()
      .filter(|rule| match rule {
        Rule::AtRule(at_rule) => {
          if let Some(AtRulePrelude::ImportPrelude(prelude)) = &at_rule.prelude {
            self.dependecies.push(ModuleDependency {
              specifier: prelude.href.as_str().unwrap().value.to_string(),
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
