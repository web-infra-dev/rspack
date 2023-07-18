use rspack_symbol::{IndirectTopLevelSymbol, StarSymbol, Symbol};
use swc_core::common::Mark;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit};
use swc_core::ecma::atoms::{js_word, JsWord};

use super::visitor::SymbolRef;
use crate::{ModuleGraph, ModuleIdentifier};

pub fn get_first_string_lit_arg(e: &CallExpr) -> Option<JsWord> {
  // we check the length at the begin of [is_require_literal_expr]
  e.args.first().and_then(|arg| match arg {
    ExprOrSpread {
      spread: None,
      expr: box Expr::Lit(Lit::Str(str)),
    } => Some(str.value.clone()),
    _ => None,
  })
}

pub fn get_require_literal(e: &CallExpr, unresolved_mark: Mark) -> Option<JsWord> {
  if e.args.len() == 1 {
    if match &e.callee {
      ident @ Callee::Expr(box Expr::Ident(Ident {
        sym: js_word!("require"),
        ..
      })) => {
        // dbg!(&ident);
        ident
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| ident.span.ctxt.outer() == unresolved_mark)
          .unwrap_or(false)
      }
      _ => false,
    } {
      get_first_string_lit_arg(e)
    } else {
      None
    }
  } else {
    None
  }
}

pub fn get_dynamic_import_string_literal(e: &CallExpr) -> Option<JsWord> {
  if e.args.len() == 1 && matches!(&e.callee, Callee::Import(_)) {
    get_first_string_lit_arg(e)
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
      SymbolRef::Direct(direct) => {
        SymbolRef::Direct(direct.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Indirect(indirect) => {
        SymbolRef::Indirect(indirect.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Star(star) => {
        SymbolRef::Star(star.convert_module_identifier_to_module_path(module_graph))
      }
      SymbolRef::Url { importer, src } => SymbolRef::Url {
        importer: importer.convert_module_identifier_to_module_path(module_graph),
        src: src.convert_module_identifier_to_module_path(module_graph),
      },
      SymbolRef::Worker { importer, src } => SymbolRef::Worker {
        importer: importer.convert_module_identifier_to_module_path(module_graph),
        src: src.convert_module_identifier_to_module_path(module_graph),
      },
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
