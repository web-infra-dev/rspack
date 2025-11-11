use rustc_hash::FxHashMap;
use swc::atoms::Wtf8Atom;
use swc_core::ecma::{
  ast::{
    Expr, Id, ImportDecl, ImportNamedSpecifier, ImportSpecifier, MemberExpr, MemberProp, Module,
    ModuleExportName,
  },
  visit::{Visit, VisitWith, noop_visit_type},
};

#[derive(Debug, Default)]
pub(crate) struct ImportMap {
  /// Map from module name to (module path, exported symbol)
  imports: FxHashMap<Id, (Wtf8Atom, Wtf8Atom)>,
  namespace_imports: FxHashMap<Id, Wtf8Atom>,
}

#[allow(unused)]
impl ImportMap {
  /// Returns true if `e` is an import of `orig_name` from `module`.
  pub fn is_import(&self, e: &Expr, module: &str, orig_name: &str) -> bool {
    match e {
      Expr::Ident(i) => {
        if let Some((i_src, i_sym)) = self.imports.get(&i.to_id()) {
          i_src == module && i_sym == orig_name
        } else {
          false
        }
      }

      Expr::Member(MemberExpr {
        obj,
        prop: MemberProp::Ident(prop),
        ..
      }) => {
        if let Expr::Ident(obj) = &**obj {
          if let Some(obj_src) = self.namespace_imports.get(&obj.to_id()) {
            obj_src == module && prop.sym == *orig_name
          } else {
            false
          }
        } else {
          false
        }
      }

      _ => false,
    }
  }

  pub fn analyze(m: &Module) -> Self {
    let mut data = ImportMap::default();

    m.visit_with(&mut Analyzer { data: &mut data });

    data
  }
}

struct Analyzer<'a> {
  data: &'a mut ImportMap,
}

impl Visit for Analyzer<'_> {
  noop_visit_type!();

  fn visit_import_decl(&mut self, import: &ImportDecl) {
    for s in &import.specifiers {
      let (local, orig_sym) = match s {
        ImportSpecifier::Named(ImportNamedSpecifier {
          local, imported, ..
        }) => match imported {
          Some(imported) => (local.to_id(), orig_name(imported)),
          _ => (local.to_id(), Wtf8Atom::from(local.sym.clone())),
        },
        ImportSpecifier::Default(s) => (s.local.to_id(), Wtf8Atom::from("default")),
        ImportSpecifier::Namespace(s) => {
          self
            .data
            .namespace_imports
            .insert(s.local.to_id(), import.src.value.clone());
          continue;
        }
      };

      self
        .data
        .imports
        .insert(local, (import.src.value.clone(), orig_sym));
    }
  }
}

fn orig_name(n: &ModuleExportName) -> Wtf8Atom {
  match n {
    ModuleExportName::Ident(v) => Wtf8Atom::from(v.sym.clone()),
    ModuleExportName::Str(v) => v.value.clone(),
  }
}
