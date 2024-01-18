use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, Lit};
use swc_core::ecma::atoms::Atom;

use super::symbol::{IndirectTopLevelSymbol, StarSymbol, Symbol};
use super::visitor::SymbolRef;
use crate::{ModuleGraph, ModuleIdentifier};

fn get_first_string(e: &CallExpr) -> Option<Atom> {
  // we check the length at the begin of [is_require_literal_expr]
  e.args.first().and_then(|arg| {
    if arg.spread.is_some() {
      None
    } else if let Some(str) = arg.expr.as_lit().and_then(|lit| match lit {
      Lit::Str(str) => Some(str),
      _ => None,
    }) {
      Some(str.value.clone())
    } else if let Some(tpl) = arg.expr.as_tpl()
      && tpl.exprs.is_empty()
      && tpl.quasis.len() == 1
      && let Some(quasis) = tpl.quasis.first()
      && let Some(cooked) = &quasis.cooked
    {
      Some(cooked.clone())
    } else {
      None
    }
  })
}

pub fn get_require_literal(e: &CallExpr, unresolved_ctxt: SyntaxContext) -> Option<Atom> {
  if e.args.len() == 1 {
    if match &e.callee {
      ident @ Callee::Expr(box Expr::Ident(Ident { sym, .. })) if sym == "require" => {
        // dbg!(&ident);
        ident
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| ident.span.ctxt == unresolved_ctxt)
          .unwrap_or(false)
      }
      _ => false,
    } {
      get_first_string(e)
    } else {
      None
    }
  } else {
    None
  }
}

pub fn get_dynamic_import_string_literal(e: &CallExpr) -> Option<Atom> {
  if e.args.len() == 1 && matches!(&e.callee, Callee::Import(_)) {
    get_first_string(e)
  } else {
    None
  }
}

/// # Panic
/// when module_identifier is not a pattern of xxx|xxxxxxxxxxxxxxxxxx
pub fn get_path_of_module_identifier<T: AsRef<str>>(path: T) -> String {
  let t = path.as_ref();
  let (_, path) = t.split_once('|').expect("Expect have `|` delimiter ");
  path.to_string()
}

pub trait ConvertModulePath {
  fn convert_module_identifier_to_module_path(self, module_graph: &ModuleGraph) -> Self;
}

impl ConvertModulePath for SymbolRef {
  fn convert_module_identifier_to_module_path(self, module_graph: &ModuleGraph) -> Self {
    match self {
      SymbolRef::Declaration(direct) => {
        SymbolRef::Declaration(direct.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Indirect(indirect) => {
        SymbolRef::Indirect(indirect.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Star(star) => {
        SymbolRef::Star(star.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Url {
        importer,
        src,
        dep_id,
      } => SymbolRef::Url {
        importer: importer.convert_module_identifier_to_module_path(module_graph),
        src: src.convert_module_identifier_to_module_path(module_graph),
        dep_id,
      },
      SymbolRef::Worker {
        importer,
        src,
        dep_id,
      } => SymbolRef::Worker {
        importer: importer.convert_module_identifier_to_module_path(module_graph),
        src: src.convert_module_identifier_to_module_path(module_graph),
        dep_id,
      },
      SymbolRef::Usage(binding, member_chain, src) => SymbolRef::Usage(
        binding,
        member_chain,
        src.convert_module_identifier_to_module_path(module_graph),
      ),
    }
  }
}

impl ConvertModulePath for Symbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    if let Some(source_path) = module_graph.normal_module_source_path_by_identifier(&self.src()) {
      self.set_src(source_path.as_ref().into());
    }
    self
  }
}

impl ConvertModulePath for IndirectTopLevelSymbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    if let Some(source_path) =
      module_graph.normal_module_source_path_by_identifier(&self.importer())
    {
      self.set_importer(source_path.as_ref().into());
    }
    if let Some(source_path) = module_graph.normal_module_source_path_by_identifier(&self.src()) {
      self.set_src(source_path.as_ref().into());
    }
    self
  }
}

impl ConvertModulePath for StarSymbol {
  fn convert_module_identifier_to_module_path(mut self, module_graph: &ModuleGraph) -> Self {
    if let Some(source_path) = module_graph.normal_module_source_path_by_identifier(&self.src()) {
      self.set_src(source_path.as_ref().into());
    }

    if let Some(source_path) =
      module_graph.normal_module_source_path_by_identifier(&self.module_ident())
    {
      self.set_module_ident(source_path.as_ref().into());
    };
    self
  }
}

impl ConvertModulePath for ModuleIdentifier {
  fn convert_module_identifier_to_module_path(self, module_graph: &ModuleGraph) -> Self {
    if let Some(source_path) = module_graph.normal_module_source_path_by_identifier(&self) {
      source_path.as_ref().into()
    } else {
      self
    }
  }
}
