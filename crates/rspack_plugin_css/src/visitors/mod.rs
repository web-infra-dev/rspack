pub mod css_assets;

use rspack_core::{ModuleDependency, ResolveKind};
use swc_common::util::take::Take;
use swc_css::{
  ast::{AtRulePrelude, Rule, Stylesheet, UrlValue},
  visit::{VisitMut, VisitMutWith},
};

#[derive(Debug, Default)]
pub struct DependencyScanner {
  pub dependencies: Vec<ModuleDependency>,
}

impl VisitMut for DependencyScanner {
  fn visit_mut_stylesheet(&mut self, n: &mut Stylesheet) {
    n.visit_mut_children_with(self);
    n.rules = n
      .rules
      .take()
      .into_iter()
      .filter(|rule| match rule {
        Rule::AtRule(at_rule) => {
          if let Some(AtRulePrelude::ImportPrelude(prelude)) = &at_rule.prelude {
            let (kind, href_string) = match &prelude.href {
              swc_css::ast::ImportPreludeHref::Url(url) => {
                let href_string = url
                  .value
                  .as_ref()
                  .map(|value| match value {
                    UrlValue::Str(str) => str.value.clone(),
                    UrlValue::Raw(raw) => raw.value.clone(),
                  })
                  .unwrap_or_default();
                (ResolveKind::UrlToken, href_string)
              }
              swc_css::ast::ImportPreludeHref::Str(str) => {
                (ResolveKind::AtImport, str.value.clone())
              }
            };
            match kind {
              ResolveKind::AtImport => {}
              ResolveKind::UrlToken => {
                // TODO: This just naive checking for http:// and https://, but it's not enough.
                // Because any scheme is valid in `ImportPreludeHref::Url`, like `url(chrome://xxxx)`
                // We need to find a better way to handle this.
                if href_string.starts_with("http://") || href_string.starts_with("https://") {
                  return true;
                }
              }
              _ => {
                unreachable!("ResolveKind in CssPlugin could either be `AtImport` or `UrlToken`")
              }
            };
            self.dependencies.push(ModuleDependency {
              specifier: href_string.to_string(),
              kind,
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
