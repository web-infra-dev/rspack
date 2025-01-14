use swc_core::atoms::Atom;
use swc_core::common::collections::AHashMap;
use swc_core::common::BytePos;
use swc_core::ecma::ast::Ident;
use swc_core::ecma::visit::{noop_visit_type, Visit};

pub struct IdentCollector {
  pub names: AHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
