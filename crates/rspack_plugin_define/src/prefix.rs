use rspack_core::ast;
use rspack_swc::swc_common::SyntaxContext;
use rspack_swc::swc_ecma_visit::Visit;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct PatMetaInfo {
  pub name: String,
  pub ctxt: SyntaxContext,
}

pub(crate) struct DefinePrefix<'a> {
  defintions: &'a HashMap<String, String>,
  /// The ident which contains in `define` and do not need to be renamed.
  pub(crate) can_not_rename: HashSet<PatMetaInfo>,
}

impl<'a> DefinePrefix<'a> {
  pub(crate) fn new(define: &'a HashMap<String, String>) -> Self {
    Self {
      defintions: define,
      can_not_rename: Default::default(),
    }
  }

  fn insert_cant_rename(&mut self, name: &str, ctxt: SyntaxContext) {
    self.defintions.contains_key(name).then(|| {
      self.can_not_rename.insert(PatMetaInfo {
        name: name.to_string(),
        ctxt,
      });
    });
  }

  pub(super) fn insert_cant_rename_from_pat(&mut self, pat: &ast::Pat) {
    match pat {
      ast::Pat::Ident(ident) => {
        self.insert_cant_rename(ident.id.sym.as_ref(), ident.id.span.ctxt);
      }
      ast::Pat::Array(array) => array.elems.iter().for_each(|ele| {
        if let Some(ele) = ele {
          self.insert_cant_rename_from_pat(ele);
        }
      }),
      ast::Pat::Rest(rest) => {
        self.insert_cant_rename_from_pat(rest.arg.as_ref());
      }
      ast::Pat::Object(obj) => {
        obj.props.iter().for_each(|prop| match prop {
          ast::ObjectPatProp::KeyValue(key_value) => {
            self.insert_cant_rename_from_pat(key_value.value.as_ref());
          }
          ast::ObjectPatProp::Assign(assign) => {
            self.insert_cant_rename(assign.key.sym.as_ref(), assign.key.span.ctxt)
          }
          ast::ObjectPatProp::Rest(rest) => self.insert_cant_rename_from_pat(rest.arg.as_ref()),
        });
      }
      ast::Pat::Assign(assign) => {
        self.insert_cant_rename_from_pat(assign.left.as_ref());
      }
      _ => unreachable!(),
    }
  }
}

impl<'a> Visit for DefinePrefix<'a> {
  // /// TODO: can't get the ident `a` in `let {a} = {a:1}`.
  // fn visit_binding_ident(&mut self, ident: &BindingIdent) {
  //   let str = ident.id.sym.as_ref();
  //   if self.defintions.contains_key(str) {
  //     self.can_not_rename.insert(str.to_string());
  //   }
  // }

  fn visit_import_decl(&mut self, import_decl: &ast::ImportDecl) {
    import_decl
      .specifiers
      .iter()
      .for_each(|specifier| match specifier {
        ast::ImportSpecifier::Default(s) => {
          self.insert_cant_rename(s.local.sym.as_ref(), s.local.span.ctxt);
        }
        ast::ImportSpecifier::Named(named) => {
          self.insert_cant_rename(named.local.sym.as_ref(), named.local.span.ctxt);
        }
        ast::ImportSpecifier::Namespace(namespace) => {
          self.insert_cant_rename(namespace.local.sym.as_ref(), namespace.local.span.ctxt);
        }
      });
  }

  fn visit_var_decl(&mut self, var_decl: &ast::VarDecl) {
    for decl in &var_decl.decls {
      self.insert_cant_rename_from_pat(&decl.name)
    }
  }
}
