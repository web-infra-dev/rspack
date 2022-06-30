pub mod css_assets;

use rspack_core::{ModuleDependency, ResolveKind};
use swc_css::{
  ast::{AtRulePrelude, Rule, Stylesheet},
  visit::VisitMut,
};

#[derive(Debug, Default)]
pub struct DependencyScanner {
  pub dependecies: Vec<ModuleDependency>,
}

impl VisitMut for DependencyScanner {
  fn visit_mut_stylesheet(&mut self, n: &mut Stylesheet) {
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
