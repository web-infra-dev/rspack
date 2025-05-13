use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_core::{
  ChunkLink, ChunkUkey, Compilation, ConcatenationScope, IdentCollector, ModuleIdentifier,
};
use rspack_javascript_compiler::ast::Ast;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet},
};
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::{Id, Ident},
    visit::{Visit, VisitWith},
  },
};

use crate::render::ConcateInfo;

pub struct SymbolRef {
  // The chunk that the symbol is defined in
  chunk: ChunkUkey,

  // The module that the symbol is defined in
  module: ModuleIdentifier,

  // The name of the symbol
  inner_name: Atom,
}

pub struct SymbolForModule {
  belong_to: ModuleIdentifier,
  alias: Option<Atom>,
}

pub struct Symbols {
  all_symbols_used: UkeyMap<ChunkUkey, HashSet<Atom>>,
  symbol_for_module: IdentifierMap<HashMap<Atom, Atom>>,
}

impl Symbols {
  pub fn prepare(
    compilation: &Compilation,
    link: UkeyMap<ChunkUkey, ChunkLink>,
    concat_infos: IdentifierMap<ConcateInfo>,
  ) -> Self {
    let mut all_symbols_used = UkeyMap::default();

    // rename export symbols
    for (chunk, chunk_link) in &link {
      let mut symbols = HashSet::default();

      // merge all used name

      all_symbols_used.insert(*chunk, symbols);
    }

    Self {
      all_symbols_used,
      symbol_for_module: IdentifierMap::default(),
    }
  }

  pub fn final_name(&mut self, chunk_ukey: ChunkUkey, module: ModuleIdentifier, ast: &Ast) {}
}
