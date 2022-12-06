use itertools::Itertools;
use rspack_core::ModuleDependency;
use swc_css::ast::{ImportHref, ImportPrelude, Stylesheet, Url, UrlValue};
use swc_css::visit::{Visit, VisitWith};

pub fn analyze_dependencies(ss: &Stylesheet) -> Vec<ModuleDependency> {
  let mut v = Analyzer {
    deps: Default::default(),
  };
  ss.visit_with(&mut v);
  v.deps = v.deps.into_iter().unique().collect();
  v.deps
}

struct Analyzer {
  deps: Vec<ModuleDependency>,
}

impl Visit for Analyzer {
  fn visit_import_prelude(&mut self, n: &ImportPrelude) {
    n.visit_children_with(self);

    match &*n.href {
      ImportHref::Url(u) => {
        if let Some(s) = &u.value {
          match &**s {
            UrlValue::Str(s) => {
              self.deps.push(ModuleDependency {
                specifier: s.value.to_string(),
                kind: rspack_core::ResolveKind::AtImportUrl,
                span: Some(n.span.into()),
              });
            }
            UrlValue::Raw(v) => {
              self.deps.push(ModuleDependency {
                specifier: v.value.to_string(),
                kind: rspack_core::ResolveKind::AtImportUrl,
                span: Some(n.span.into()),
              });
            }
          }
        }
      }
      ImportHref::Str(s) => {
        self.deps.push(ModuleDependency {
          specifier: s.value.to_string(),
          kind: rspack_core::ResolveKind::AtImport,
          span: Some(n.span.into()),
        });
      }
    }
  }

  fn visit_url(&mut self, u: &Url) {
    u.visit_children_with(self);

    match &u.value {
      Some(box UrlValue::Str(s)) => {
        self.deps.push(ModuleDependency {
          specifier: s.value.to_string(),
          kind: rspack_core::ResolveKind::UrlToken,
          span: Some(u.span.into()),
        });
      }
      Some(box UrlValue::Raw(r)) => {
        self.deps.push(ModuleDependency {
          specifier: r.value.to_string(),
          kind: rspack_core::ResolveKind::UrlToken,
          span: Some(u.span.into()),
        });
      }
      None => {}
    };
  }
}
