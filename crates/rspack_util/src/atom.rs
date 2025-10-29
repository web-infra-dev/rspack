use swc_core::ecma::ast::ModuleExportName;

pub type Atom = swc_core::atoms::Atom;

pub trait ModuleExportNameExt {
  fn atom_ref(&self) -> &Atom;
}

impl ModuleExportNameExt for ModuleExportName {
  fn atom_ref(&self) -> &Atom {
    match self {
      ModuleExportName::Ident(ident) => &ident.sym,
      ModuleExportName::Str(s) => s
        .value
        .as_atom()
        .expect("ModuleExportName should be a valid utf8"),
    }
  }
}
