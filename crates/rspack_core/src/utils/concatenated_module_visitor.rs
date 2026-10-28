use rspack_intern::Atom;
use rspack_sources::ReplaceSource;
use rspack_util::{SpanExt, fx_hash::FxIndexMap};
use swc_next_ecma_ast::{Span, SymbolId};

/// An owned declaration or reference position retained after dropping the AST.
#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub name: Atom,
  pub span: Span,
  pub shorthand: bool,
}

/// A declaration or reference of a resolved symbol; its name is held by the group.
#[derive(Debug)]
pub struct ConcatenatedModuleReference {
  pub span: Span,
  pub shorthand: bool,
}

/// The symbols whose names semantic requires us to rename together.
#[derive(Debug)]
pub struct ConcatenatedModuleSymbol {
  pub name: Atom,
  pub symbols: Vec<SymbolId>,
  pub top_level: bool,
  pub references: Vec<ConcatenatedModuleReference>,
}

/// Owned renaming facts extracted from semantic for module rendering and caches.
/// Each key is a representative SymbolId; linked symbols and their occurrences
/// are collected once per rename group, including declarations without uses.
#[derive(Debug, Default)]
pub struct ConcatenatedModuleAnalysis {
  pub binding_to_ref: FxIndexMap<SymbolId, ConcatenatedModuleSymbol>,
  pub global_scope_ident: Vec<ConcatenatedModuleIdent>,
}

impl ConcatenatedModuleAnalysis {
  pub fn used_names(&self) -> impl Iterator<Item = &Atom> {
    self
      .binding_to_ref
      .values()
      .map(|symbol| &symbol.name)
      .chain(
        self
          .global_scope_ident
          .iter()
          .map(|reference| &reference.name),
      )
  }

  pub fn symbol_groups(&self) -> impl Iterator<Item = SymbolId> + '_ {
    self.binding_to_ref.keys().copied()
  }

  pub fn symbol_name(&self, symbol: SymbolId) -> &Atom {
    &self.binding_to_ref[&symbol].name
  }

  pub fn is_top_level(&self, symbol: SymbolId) -> bool {
    self.binding_to_ref[&symbol].top_level
  }

  pub fn unresolved_references(&self) -> impl Iterator<Item = &ConcatenatedModuleIdent> {
    self.global_scope_ident.iter()
  }

  pub fn rename_symbol(&self, symbol: SymbolId, name: &str, source: &mut ReplaceSource) {
    for ident in &self.binding_to_ref[&symbol].references {
      if ident.shorthand {
        source.insert(ident.span.real_hi(), format!(": {name}"), None);
      } else {
        source.replace(
          ident.span.real_lo(),
          ident.span.real_hi(),
          name.to_owned(),
          None,
        );
      }
    }
  }
}
