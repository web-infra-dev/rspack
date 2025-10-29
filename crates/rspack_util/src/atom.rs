use std::borrow::Borrow;

use swc_core::{atoms::Wtf8Atom, ecma::ast::ModuleExportName};

pub type Atom = swc_core::atoms::Atom;

pub trait ModuleExportNameExt {
  fn wtf8(&self) -> &Wtf8Atom;
}

impl ModuleExportNameExt for ModuleExportName {
  fn wtf8(&self) -> &Wtf8Atom {
    match self {
      ModuleExportName::Ident(ident) => ident.sym.borrow(),
      ModuleExportName::Str(str) => &str.value,
    }
  }
}
