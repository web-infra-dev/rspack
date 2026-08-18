use rustc_hash::FxHashSet;
use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{
      CallExpr, Callee, Decl, ExportAll, ExportDecl, ExportDefaultDecl, ExportDefaultExpr,
      ExportSpecifier, Expr, ImportDecl, Lit, ModuleExportName, NamedExport, ObjectPatProp, Pat,
      TsExternalModuleRef, TsImportType,
    },
    visit::{Visit, VisitWith},
  },
};

fn module_export_name_to_atom(name: &ModuleExportName) -> Atom {
  name.atom().into_owned()
}

fn visit_pat_binding_names(pat: &Pat, f: &mut impl FnMut(Atom)) {
  match pat {
    Pat::Ident(ident) => f(Atom::from(ident.id.sym.as_str())),
    Pat::Array(array) => {
      for elem in array.elems.iter().flatten() {
        visit_pat_binding_names(elem, f);
      }
    }
    Pat::Object(object) => {
      for prop in &object.props {
        match prop {
          ObjectPatProp::KeyValue(prop) => visit_pat_binding_names(&prop.value, f),
          ObjectPatProp::Assign(prop) => f(Atom::from(prop.key.id.sym.as_str())),
          ObjectPatProp::Rest(prop) => visit_pat_binding_names(&prop.arg, f),
        }
      }
    }
    Pat::Assign(assign) => visit_pat_binding_names(&assign.left, f),
    Pat::Rest(rest) => visit_pat_binding_names(&rest.arg, f),
    Pat::Expr(_) | Pat::Invalid(_) => {}
  }
}

#[derive(Debug)]
pub struct ImportsExportsCollector<'a> {
  exports: &'a mut FxHashSet<Atom>,
  imported_modules: &'a mut FxHashSet<Atom>,
}

impl<'a> ImportsExportsCollector<'a> {
  pub fn new(exports: &'a mut FxHashSet<Atom>, imported_modules: &'a mut FxHashSet<Atom>) -> Self {
    Self {
      exports,
      imported_modules,
    }
  }

  fn add_imported_module(&mut self, value: String) {
    self.imported_modules.insert(Atom::from(value));
  }
}

impl Visit for ImportsExportsCollector<'_> {
  fn visit_import_decl(&mut self, node: &ImportDecl) {
    self.add_imported_module(node.src.value.to_string_lossy().into_owned());
  }

  fn visit_named_export(&mut self, node: &NamedExport) {
    if let Some(src) = &node.src {
      self.add_imported_module(src.value.to_string_lossy().into_owned());
    }

    for specifier in &node.specifiers {
      match specifier {
        ExportSpecifier::Namespace(specifier) => {
          self
            .exports
            .insert(module_export_name_to_atom(&specifier.name));
        }
        ExportSpecifier::Default(specifier) => {
          self
            .exports
            .insert(Atom::from(specifier.exported.sym.as_str()));
        }
        ExportSpecifier::Named(specifier) => {
          let exported = specifier.exported.as_ref().unwrap_or(&specifier.orig);
          self.exports.insert(module_export_name_to_atom(exported));
        }
      }
    }
  }

  fn visit_export_all(&mut self, node: &ExportAll) {
    self.add_imported_module(node.src.value.to_string_lossy().into_owned());
  }

  fn visit_export_decl(&mut self, node: &ExportDecl) {
    match &node.decl {
      Decl::Class(decl) => {
        self.exports.insert(Atom::from(decl.ident.sym.as_str()));
      }
      Decl::Fn(decl) => {
        self.exports.insert(Atom::from(decl.ident.sym.as_str()));
      }
      Decl::Var(decl) => {
        for declarator in &decl.decls {
          visit_pat_binding_names(&declarator.name, &mut |name| {
            self.exports.insert(name);
          });
        }
      }
      Decl::TsInterface(decl) => {
        self.exports.insert(decl.id.sym.clone());
      }
      Decl::TsTypeAlias(decl) => {
        self.exports.insert(decl.id.sym.clone());
      }
      Decl::TsEnum(decl) => {
        self.exports.insert(decl.id.sym.clone());
      }
      Decl::TsModule(decl) => {
        if let Some(ident) = decl.id.as_ident() {
          self.exports.insert(ident.sym.clone());
        }
      }
      _ => {}
    }

    node.visit_children_with(self);
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    self.exports.insert("default".into());
    node.visit_children_with(self);
  }

  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    self.exports.insert("default".into());
    node.visit_children_with(self);
  }

  fn visit_ts_import_type(&mut self, node: &TsImportType) {
    self.add_imported_module(node.arg.value.to_string_lossy().into_owned());
    node.visit_children_with(self);
  }

  fn visit_ts_external_module_ref(&mut self, node: &TsExternalModuleRef) {
    self.add_imported_module(node.expr.value.to_string_lossy().into_owned());
  }

  fn visit_call_expr(&mut self, node: &CallExpr) {
    if matches!(node.callee, Callee::Import(_))
      && let Some(arg) = node.args.first()
      && let Expr::Lit(Lit::Str(src)) = arg.expr.as_ref()
    {
      self.add_imported_module(src.value.to_string_lossy().into_owned());
    }

    node.visit_children_with(self);
  }
}
